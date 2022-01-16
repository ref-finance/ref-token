//! Account is information per user about their locking balances and ballots.
//!

use crate::utils::{ext_self, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER, NO_DEPOSIT};
use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, log, AccountId, Balance, Promise, PromiseOrValue, PromiseResult};

use crate::proposals::Vote;

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VAccount {
    Current(Account),
}

impl VAccount {
    /// Upgrades from other versions to the currently used version.
    pub fn into_current(self) -> Account {
        match self {
            VAccount::Current(account) => account,
        }
    }
}

impl From<Account> for VAccount {
    fn from(account: Account) -> Self {
        VAccount::Current(account)
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountVote {
    pub vote: Vote,
    pub amount: Balance,
}

/// Account information.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    /// The amount of base token locked
    pub locking_amount: Balance,
    /// The amount of ballots the account holds
    pub ballot_amount: Balance,
    /// unlocking session id, unlocking at the begining of this session
    pub unlocking_session_id: u32,
    /// Record proposal voting info
    pub proposals: LookupMap<u32, AccountVote>,
}

impl Account {
    pub(crate) fn add_locking(
        &mut self,
        locking_amount: Balance,
        ballot_amount: Balance,
        unlocking_session_id: u32,
    ) {
        self.locking_amount += locking_amount;
        self.ballot_amount += ballot_amount;
        self.unlocking_session_id = unlocking_session_id;
    }

    pub(crate) fn remove_locking(&mut self) -> Balance {
        if self.ballot_amount == 0 {
            let amount = self.locking_amount;
            self.locking_amount = 0;
            amount
        } else {
            0
        }
    }

    /// return account's current valid ballot
    pub(crate) fn sync_ballot(&self, cur_session_id: u32) -> Balance {
        if cur_session_id >= self.unlocking_session_id {
            0
        } else {
            self.ballot_amount
        }
    }

    /// update account's valid ballot according to current session id
    pub(crate) fn touch(&mut self, cur_session_id: u32) {
        self.ballot_amount = self.sync_ballot(cur_session_id);
    }
}

impl Contract {
    /// lasts = 0 means APPEND mode, otherwise NEW mode with session numbers to lock,
    /// In APPEND mode, user must have valid ballots,
    /// In NEW mode, user must have 0 ballots,but can have unlocked tokens
    fn internal_lock(&mut self, account_id: &AccountId, locking_amount: Balance, lasts: u32) {
        let current_state = self.data();
        let mut account = current_state
            .accounts
            .get(account_id)
            .map(|va| va.into_current())
            .expect("ERR_USER_NOT_REGISTER");
        let current_session_info = current_state.sessions[current_state.cur_session];

        account.touch(self.get_cur_session_id());

        let current_session_remaining_days = nano_to_day(SESSION_INTERMAL)
            - nano_to_day(
                env::block_timestamp()
                    - current_state.genesis_timestamp
                    - current_session_info.session_id as u64 * SESSION_INTERMAL,
            );

        let (ballot_amount, unlocking_session_id) = {
            if lasts == 0 {
                // APPEND mode, verify non-zero ballot
                assert!(account.ballot_amount != 0, "ERR_NO_RUNNING_LOCKING");
                (
                    calculate_ballots(
                        current_session_remaining_days, 
                        locking_amount, 
                        account.unlocking_session_id - current_session_info.session_id),
                    account.unlocking_session_id
                )
            } else {
                // NEW mode, verify zero ballot
                assert_eq!(account.ballot_amount, 0, "ERR_EXIST_RUNNING_LOCKING");
                (
                    calculate_ballots(
                        current_session_remaining_days, 
                        account.locking_amount + locking_amount, 
                        lasts),
                    current_session_info.session_id + lasts
                )
            }
        };
        
        // locate end_session (array index)
        let end_session = (current_state.cur_session
            + (unlocking_session_id - 1 - current_session_info.session_id) as usize)
            % MAX_SESSIONS;

        // update account
        account.add_locking(locking_amount, ballot_amount, unlocking_session_id);
        self.data_mut().cur_total_ballot += ballot_amount;
        if lasts == 0 {
            // auto update user involved proposal votes
            if let Some(proposal_ids) = self
                .data()
                .proposal_ids_in_sessions
                .get(current_session_info.session_id as u64)
            {
                for proposal_id in proposal_ids {
                    if let Some(mut account_vote) = account.proposals.get(&proposal_id) {
                        let append_amount = self.internal_append_vote(
                            proposal_id,
                            &account_vote.vote,
                            &ballot_amount,
                        );
                        account_vote.amount += append_amount;
                        account.proposals.insert(&proposal_id, &account_vote);
                    }
                }
            }
            log!(
                "User {} appends locking with {} token got {} ballots, total {} ballots",
                account_id,
                locking_amount,
                ballot_amount,
                account.ballot_amount,
            );
        } else {
            log!(
                "User {} starts new locking with total {} token got {} ballots,  unlocking_session_id : {}",
                account_id,
                account.locking_amount,
                ballot_amount,
                unlocking_session_id
            );
        }

        self.data_mut().sessions[end_session].expire_amount += ballot_amount;
        self.data_mut().accounts.insert(account_id, &account.into());
    }

    fn internal_withdraw(&mut self, account_id: &AccountId) -> Balance {
        let current_state = self.data();
        let mut account = current_state
            .accounts
            .get(account_id)
            .map(|va| va.into_current())
            .expect("ERR_USER_NOT_REGISTER");
        account.touch(self.get_cur_session_id());
        let amount = account.remove_locking();
        assert!(amount > 0, "ERR_NOTHING_CAN_BE_WITHDRAW");
        self.data_mut().accounts.insert(account_id, &account.into());
        amount
    }

    pub(crate) fn internal_register_account(&mut self, account_id: &AccountId) {
        self.data_mut().accounts.insert(
            account_id,
            &Account {
                locking_amount: 0,
                ballot_amount: 0,
                unlocking_session_id: 0,
                proposals: LookupMap::new(StorageKeys::AccountProposals {
                    account_id: account_id.clone(),
                }),
            }
            .into(),
        );
    }

    /// user first vote for given proposal
    /// return non-zero ballot power
    /// panic if following:
    /// * user not registered
    /// * user has no valid ballots
    /// * user already voted
    pub(crate) fn internal_account_vote(
        &mut self,
        account_id: &AccountId,
        proposal_id: u32,
        vote: &Vote,
    ) -> Balance {
        let mut account = self
            .data()
            .accounts
            .get(account_id)
            .map(|va| va.into_current())
            .expect("ERR_USER_NOT_REGISTER");
        account.touch(self.get_cur_session_id());
        assert!(account.ballot_amount > 0, "ERR_NO_BALLOTS");
        assert!(
            !account.proposals.contains_key(&proposal_id),
            "ERR_ALREADY_VOTED"
        );
        let account_vote = AccountVote {
            vote: vote.clone(),
            amount: account.ballot_amount,
        };
        account.proposals.insert(&proposal_id, &account_vote);
        self.data_mut().accounts.insert(account_id, &account.into());
        account_vote.amount
    }
}

#[near_bindgen]
impl Contract {
    /// withdraw unlocked token back to the predecessor account.
    /// Requirements:
    /// * The predecessor account should be registered.
    /// * Requires attached deposit of exactly 1 yoctoNEAR.
    /// Return:
    /// * Promise or Panic if nothing unlocked
    #[payable]
    pub fn withdraw(&mut self) -> Promise {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();
        self.fresh_sessions();
        let unlocked = self.internal_withdraw(&account_id);
        log!("Withdraw {} token back to {}", unlocked, account_id);

        self.data_mut().cur_lock_amount -= unlocked;
        ext_fungible_token::ft_transfer(
            account_id.clone(),
            U128(unlocked),
            None,
            &self.data().locked_token,
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::callback_post_withdraw(
            account_id.clone(),
            U128(unlocked),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
    }

    #[private]
    pub fn callback_post_withdraw(&mut self, sender_id: AccountId, amount: U128) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ERR: expected 1 promise result from withdraw"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                // This reverts the changes from withdraw function.
                // If account doesn't exit, the token stays in contract.
                if self.data().accounts.contains_key(&sender_id) {
                    let mut account = self
                        .data()
                        .accounts
                        .get(&sender_id)
                        .map(|va| va.into_current())
                        .unwrap();
                    account.locking_amount += amount.0;
                    self.data_mut().cur_lock_amount += amount.0;
                    self.data_mut().accounts.insert(&sender_id, &account.into());

                    env::log(
                        format!(
                            "Account {} withdraw {} token failed and reverted.",
                            sender_id, amount.0
                        )
                        .as_bytes(),
                    );
                } else {
                    env::log(
                        format!(
                            "Account {} has unregisterd. withdraw {} token goes to contract.",
                            sender_id, amount.0
                        )
                        .as_bytes(),
                    );
                }
            }
        };
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // sync point
        self.fresh_sessions();

        let token_in = env::predecessor_account_id();
        let amount: Balance = amount.into();
        assert_eq!(token_in, self.data().locked_token, "ERR_ILLEGAL_TOKEN");

        if msg.is_empty() {
            // user append locking
            self.internal_lock(sender_id.as_ref(), amount, 0);
        } else {
            // new locking
            let locking_period = msg.parse::<u32>().expect("ERR_ILLEGAL_MSG");
            assert!(locking_period > 0, "ERR_ILLEGAL_MSG");
            assert!((locking_period as usize) <= MAX_SESSIONS, "ERR_ILLEGAL_MSG");
            self.internal_lock(sender_id.as_ref(), amount, locking_period);
        }
        self.data_mut().cur_lock_amount += amount;
        PromiseOrValue::Value(U128(0))
    }
}

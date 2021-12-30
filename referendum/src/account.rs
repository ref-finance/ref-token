//! Account is information per user about their locking balances and ballots.
//! 

use crate::*; 
use crate::utils::{ext_self, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER, NO_DEPOSIT};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{Balance, AccountId, assert_one_yocto, PromiseOrValue, log, PromiseResult};
use near_sdk::json_types::U128;

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
    /// unlocking session id
    pub unlocking_session_id: u32,
    /// Record proposal voting info, when ballot_amount changes, should update this one and global proposal vote info
    pub proposals: LookupMap<u64, AccountVote>,
}

impl Account {
    pub(crate) fn add_locking(&mut self, locking_amount: Balance, ballot_amount: Balance, unlocking_session_id: u32) {
        self.locking_amount += locking_amount;
        self.ballot_amount += ballot_amount;
        self.unlocking_session_id = unlocking_session_id;
    }

    pub(crate) fn unlock(&mut self, cur_session_id: u32) -> Balance {
        if cur_session_id > self.unlocking_session_id {
            let amount =  self.locking_amount;
            self.locking_amount = 0;
            self.ballot_amount = 0;
            self.unlocking_session_id = 0;
            amount
        } else {
            0
        }
    }
}


impl Contract {
    /// lasts = 0 means append
    fn internal_lock(&mut self, account_id: &AccountId, locking_amount: Balance, lasts: u32) {
        let current_state = self.data();
        let mut account = current_state.accounts.get(account_id).map(|va| va.into_current()).expect("ERR_USER_NOT_REGISTER");
        let current_session_info = current_state.sessions[current_state.cur_session];

        if lasts == 0 {
            // verify append has valid running locking
            assert!(account.locking_amount != 0, "Append stake: ERR_ACCOUNT_HAS_NO_RUNNING_LOCKING");
            assert!(account.unlocking_session_id >= current_session_info.session_id, "Append stake: ERR_BALLOTS_EXPIRE_NOT_UNLOCK");
        }else{
            // vierify new lock has no running locking
            assert!(account.locking_amount == 0, "New stake: ERR_ACCOUNT_HAS_VALID_RUNNING_LOCKING_OR_BALLOTS_EXPIRE_NOT_UNLOCK");
        }

        let mut end_session: usize = 0;
        let locking_period = if lasts == 0 {account.unlocking_session_id - current_session_info.session_id + 1} else { lasts };

        // calculate ballots
        let full_session_ballots = locking_amount * (locking_period - 1) as u128;
        let current_session_remaining_days = nano_to_day(SESSION_INTERMAL) - 
            nano_to_day(env::block_timestamp() - current_state.genesis_timestamp - current_session_info.session_id as u64 * SESSION_INTERMAL);
        let part_session_ballots = (U256::from(locking_amount) * U256::from(current_session_remaining_days) / U256::from(nano_to_day(SESSION_INTERMAL))).as_u128();
        let ballot_amount = full_session_ballots + part_session_ballots;

        // locate end_session (array index) and end_session_id
        let end_session_id: u32 = if lasts == 0 {account.unlocking_session_id} else {current_session_info.session_id + (locking_period - 1)};
        for i in 0..MAX_SESSIONS {
            let idx = (i + current_state.cur_session) % MAX_SESSIONS;
            if current_state.sessions[idx].session_id == end_session_id {
                end_session = idx;
                break;
            }
        }
        // update the account
        account.add_locking(locking_amount, ballot_amount, end_session_id);
        // update the session
        self.data_mut().sessions[end_session].expire_amount += ballot_amount;
        // update the ballot
        self.data_mut().cur_total_ballot += ballot_amount;

        if lasts == 0{
            if let Some(proposal_ids) = self.data().proposal_ids_in_sessions.get(current_session_info.session_id as u64){
                for proposal_id in proposal_ids{
                    if let Some(mut account_vote) = account.proposals.get(&proposal_id){
                        let append_amount =  self.internal_append_vote(proposal_id, &account_vote.vote, &ballot_amount);
                        account_vote.amount += append_amount;
                        account.proposals.insert(&proposal_id, &account_vote);
                    }
                }
            }
        }
        
        self.data_mut().accounts.insert(account_id, &account.into());
        
        log!(
            "User {} {} {} ballots,  unlocking_session_id : {}",
            account_id,
            if lasts == 0 {"append stake"} else {"new stake"},
            ballot_amount,
            end_session_id
        );
    }

    fn internal_unlock(&mut self, account_id: &AccountId) -> Balance {
        let current_state = self.data();
        let cur_session_id = current_state.sessions[current_state.cur_session].session_id; 
        let mut account = current_state.accounts.get(account_id).map(|va| va.into_current()).expect("ERR_USER_NOT_REGISTER");
        let amount = account.unlock(cur_session_id);
        self.data_mut().accounts.insert(account_id, &account.into());
        amount
    }

    pub(crate) fn internal_register_account(&mut self, account_id: &AccountId) {
        self.data_mut().accounts.insert(account_id, &Account{
            locking_amount: 0,
            ballot_amount: 0,
            unlocking_session_id: 0,
            proposals: LookupMap::new(StorageKeys::AccountProposals{
                account_id: account_id.clone()
            })
        }.into());
    }
    pub(crate) fn internal_vote(&mut self, account_id: &AccountId, proposal_id: u64, vote: &Vote) -> Balance {
        let mut account = self.data().accounts.get(account_id).map(|va| va.into_current()).expect("ERR_USER_NOT_REGISTER");
        assert!(!account.proposals.contains_key(&proposal_id), "ERR_ALREADY_VOTED");
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

    /// unslock token and send assets back to the predecessor account.
    /// Requirements:
    /// * The predecessor account should be registered.
    /// * Requires attached deposit of exactly 1 yoctoNEAR.
    #[payable]
    pub fn unlock(&mut self) -> PromiseOrValue<u8> {

        assert_one_yocto();
        // sync point
        self.fresh_sessions();

        let account_id = env::predecessor_account_id();

        let unlocked = self.internal_unlock(&account_id);

        if unlocked > 0 {
            log!("Unlock {} token back to {}", unlocked, account_id);
            ext_fungible_token::ft_transfer(
                account_id.clone(),
                U128(unlocked),
                None,
                &self.data().locked_token,
                1,
                GAS_FOR_FT_TRANSFER,
            )
            .then(ext_self::callback_post_unlock(
                account_id.clone(),
                U128(unlocked),
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_RESOLVE_TRANSFER,
            )).into()
        } else {
            PromiseOrValue::Value(0)
        }
    }

    #[private]
    pub fn callback_post_unlock(
        &mut self,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Err: expected 1 promise result from unstake"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                // This reverts the changes from unlock function.
                // If account doesn't exit, the unlock token stay in contract.
                if self.data().accounts.contains_key(&sender_id) {
                    let mut account = self.data().accounts.get(&sender_id).map(|va| va.into_current()).unwrap();
                    account.locking_amount += amount.0;
                    self.data_mut().accounts.insert(&sender_id, &account.into());

                    env::log(
                        format!(
                            "Account {} unlock failed and reverted.",
                            sender_id
                        )
                        .as_bytes(),
                    );
                } else {
                    env::log(
                        format!(
                            "Account {} has unregisterd. unlocking token goes to contract.",
                            sender_id
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
            log!("append lock {} token", amount);
            self.internal_lock(sender_id.as_ref(), amount, 0);
        } else {
            // new locking
            let locking_period = msg.parse::<u32>().expect("ERR_ILLEGAL_MSG");
            assert!(locking_period > 0, "ERR_ILLEGAL_MSG");
            assert!((locking_period as usize) <= MAX_SESSIONS, "ERR_ILLEGAL_MSG");

            log!("New lock {} token with {} sessions", amount, msg);
            self.internal_lock(sender_id.as_ref(), amount, locking_period);
            
        }
        PromiseOrValue::Value(U128(0))
    }
}
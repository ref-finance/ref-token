//! Account is information per user about their locking balances and ballots.
//! 

use crate::*; 
use crate::utils::{ext_self, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER, NO_DEPOSIT};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{Balance, AccountId, assert_one_yocto, PromiseOrValue, log, PromiseResult};
use near_sdk::json_types::U128;

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

/// Account information.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    /// The amount of base token locked
    pub locking_amount: Balance,
    /// The amount of ballots the account holds
    pub ballot_amount: Balance,
    /// unlocking session id
    pub unlocking_session_id: u32,
    // TODO: record proposal voting info, when ballot_amount changes, should update this one and global proposal vote info
    // pub proposals: HashMap<proposal_id: u32, info: {Aye/Nay, Balance}>,
}

impl Account {
    pub(crate) fn add_locking(&mut self, locking_amount: Balance, ballot_amount: Balance, unlocking_session_id: u32) {
        self.locking_amount += locking_amount;
        self.ballot_amount += ballot_amount;
        self.unlocking_session_id = unlocking_session_id;
    }

    pub(crate) fn unlock(&mut self, cur_session_id: u32) -> Balance {
        if cur_session_id >= self.unlocking_session_id {
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
        // TODO: 
        // calculate ballots
        // locate end_session (array index) and end_session_id
        // verify append has valid running locking
        // vierify new lock has no running locking
        let ballot_amount: Balance = 0;
        let end_session: usize = 0;
        let end_session_id: u32 = 0;

        // update the account
        let mut account = self.data().accounts.get(account_id).map(|va| va.into_current()).expect("ERR_USER_NOT_REGISTER");
        account.add_locking(locking_amount, ballot_amount, end_session_id);
        self.data_mut().accounts.insert(account_id, &account.into());
        // update the session
        self.data_mut().sessions[end_session].expire_amount += ballot_amount;
        // update the ballot
        self.data_mut().cur_total_ballot += ballot_amount;
        // TODO: add log
    }

    fn internal_unlock(&mut self, account_id: &AccountId) -> Balance {
        // TODO: get cur session id
        let cur_session_id = 0; 
        let mut account = self.data().accounts.get(account_id).map(|va| va.into_current()).expect("ERR_USER_NOT_REGISTER");
        let amount = account.unlock(cur_session_id);
        self.data_mut().accounts.insert(account_id, &account.into());
        // TODO: call withdraw to send unlock token back to user
        amount
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
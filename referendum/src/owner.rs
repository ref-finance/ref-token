//! Implement all the relevant logic for owner of this contract.

use near_sdk::json_types::U128;
use crate::*;


#[near_bindgen]
impl Contract {
    /// Change owner. Only can be called by owner.
    pub fn set_owner(&mut self, owner_id: ValidAccountId) {
        self.assert_owner();
        self.data_mut().owner_id = owner_id.as_ref().clone();
    }

    /// Get the owner of this account.
    pub fn get_owner(&self) -> AccountId {
        self.data().owner_id.clone()
    }

    pub fn modify_genesis_timestamp(&mut self, genesis_timestamp_in_sec: u32) {
        self.assert_owner();
        let genesis_ts = sec_to_nano(genesis_timestamp_in_sec);
        assert!(
            env::block_timestamp() <= self.data().genesis_timestamp,
            "ERR_HAS_LAUNCHED"
        );
        assert!(genesis_ts > env::block_timestamp(), "ERR_ILLEGAL_GENESIS_TIME");
        self.data_mut().genesis_timestamp = genesis_ts;
    }

    pub fn modify_endorsement_amount(&mut self, amount: U128) {
        self.assert_owner();
        let amount: Balance = amount.into();
        assert!(amount > 0, "ERR_MUST_HAVE_ENDORSEMENT");
        self.data_mut().lock_amount_per_proposal = amount;
    }

    pub fn modify_nonsense_threshold(&mut self, threshold: Rational) {
        self.assert_owner();
        assert!(threshold.is_valid(), "ERR_ILLEGAL_THRESHOLD");
        self.data_mut().nonsense_threshold = threshold;
    }

    pub fn modify_vote_policy(&mut self, vote_policy: VotePolicy) {
        self.assert_owner();
        match &vote_policy {
            VotePolicy::Relative(l, j) => {
                assert!(l.is_valid(), "ERR_ILLEGAL_THRESHOLD");
                assert!(j.is_valid(), "ERR_ILLEGAL_THRESHOLD");
                if let Some(elem) = self.data_mut().vote_policy.get_mut(0) {
                    *elem = vote_policy;
                }
            },
            VotePolicy::Absolute(p, f) => {
                assert!(p.is_valid(), "ERR_ILLEGAL_THRESHOLD");
                assert!(f.is_valid(), "ERR_ILLEGAL_THRESHOLD");
                if let Some(elem) = self.data_mut().vote_policy.get_mut(1) {
                    *elem = vote_policy;
                }
            },
        }
    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.data().owner_id,
            "ERR_NOT_ALLOWED"
        );
    }

    pub(crate) fn assert_launch(&self) {
        assert!(
            env::block_timestamp() > self.data().genesis_timestamp,
            "ERR_NOT_LAUNCHED"
        );
    }

    /// Migration function.
    /// For next version upgrades, change this function.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let prev: Contract = env::state_read().expect("ERR_NOT_INITIALIZED");
        prev
    }
}


#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::env::BLOCKCHAIN_INTERFACE;
    use near_sdk::Gas;

    use super::*;

    const BLOCKCHAIN_INTERFACE_NOT_SET_ERR: &str = "Blockchain interface not set.";

    /// Gas for calling migration call.
    pub const GAS_FOR_MIGRATE_CALL: Gas = 5_000_000_000_000;

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub extern "C" fn upgrade() {
        env::setup_panic_hook();
        env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().into_bytes();
        let method_name = "migrate".as_bytes().to_vec();
        unsafe {
            BLOCKCHAIN_INTERFACE.with(|b| {
                // Load input into register 0.
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .input(0);
                let promise_id = b
                    .borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
                let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_MIGRATE_CALL;
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_action_function_call(
                        promise_id,
                        method_name.len() as _,
                        method_name.as_ptr() as _,
                        0 as _,
                        0 as _,
                        0 as _,
                        attached_gas,
                    );
            });
        }
    }

}

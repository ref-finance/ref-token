use crate::*;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

use std::convert::TryInto;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{assert_one_yocto, env, near_bindgen, Promise};

#[near_bindgen]
impl StorageManagement for Contract {
    #[allow(unused_variables)]
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.assert_launch();
        let amount = env::attached_deposit();
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());
        let min_balance = self.storage_balance_bounds().min.0;
        let already_registered = self.data().accounts.contains_key(&account_id);
        if amount < min_balance && !already_registered {
            env::panic(b"ERR_DEPOSIT_LESS_THAN_MIN_STORAGE");
        }
        if already_registered {
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            self.internal_register_account(&account_id);
            self.data_mut().account_number += 1;
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.storage_balance_of(account_id.try_into().unwrap())
            .unwrap()
    }

    #[allow(unused_variables)]
    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        env::panic(b"ERR_NO_STORAGE_CAN_WITHDRAW");
    }

    #[allow(unused_variables)]
    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        self.assert_launch();
        let current_state = self.data();
        let account_id = env::predecessor_account_id();
        if let Some(VAccount::Current(account)) = current_state.accounts.get(&account_id) {
            assert!(
                account.locking_amount == 0,
                "ERR_ACCOUNT_NOT_UNLOCK"
            );
            self.data_mut().accounts.remove(&account_id);
            let number = self.data().account_number.checked_sub(1).unwrap_or(0);
            self.data_mut().account_number = number;
            Promise::new(account_id.clone()).transfer(STORAGE_BALANCE_MIN_BOUND);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(STORAGE_BALANCE_MIN_BOUND),
            max: None,
        }
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        if self.data().accounts.contains_key(&account_id.into()) {
            Some(StorageBalance {
                total: U128(STORAGE_BALANCE_MIN_BOUND),
                available: U128(0),
            })
        }else{
            None
        }
    }
}
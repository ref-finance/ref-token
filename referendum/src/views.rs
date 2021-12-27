//! View functions for the contract.

use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractMetadata {
    pub owner_id: AccountId,
    pub locked_token: AccountId,
    pub genesis_timestamp: u64,
    pub cur_session: usize,
    pub cur_total_ballot: U128,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountInfo {
    pub locking_amount: U128,
    pub ballot_amount: U128,
    pub unlocking_session_id: u32,
}

impl From<VAccount> for AccountInfo {
    fn from(vacc: VAccount) -> Self {
        match vacc {
            VAccount::Current(acc) => {
                Self {
                    locking_amount: acc.locking_amount.into(),
                    ballot_amount: acc.ballot_amount.into(),
                    unlocking_session_id: acc.unlocking_session_id,
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct SessionState {
    pub session_id: u32,
    pub expire_amount: U128,
}

impl From<SessionInfo> for SessionState {
    fn from(session_info: SessionInfo) -> Self {
        Self {
            session_id: session_info.session_id,
            expire_amount: session_info.expire_amount.into(),
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Return contract basic info
    pub fn contract_metadata(&self) -> ContractMetadata {
        let current_state = self.data();
        ContractMetadata {
            owner_id: current_state.owner_id.clone(),
            locked_token: current_state.locked_token.clone(),
            genesis_timestamp: current_state.genesis_timestamp,
            cur_session: current_state.cur_session,
            cur_total_ballot: current_state.cur_total_ballot.into(),
        }
    }

    pub fn get_user_base_info(&self, account_id: ValidAccountId) -> AccountInfo {
        self.data().accounts.get(account_id.as_ref()).unwrap().into()
    }

    pub fn get_session_state(&self, session_idx: usize) -> SessionState {
        self.data().sessions[session_idx].into()
    }
}
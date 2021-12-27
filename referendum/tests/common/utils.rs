#![allow(unused)] 
use near_sdk_sim::{ExecutionResult, view};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use near_sdk::AccountId;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountInfo {
    pub locking_amount: U128,
    pub ballot_amount: U128,
    pub unlocking_session_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct SessionState {
    pub session_id: u32,
    pub expire_amount: U128,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractMetadata {
    pub owner_id: AccountId,
    pub locked_token: AccountId,
    pub genesis_timestamp: u64,
    pub cur_session: usize,
    pub cur_total_ballot: U128,
}

pub fn get_error_count(r: &ExecutionResult) -> u32 {
    r.promise_errors().len() as u32
}

pub fn get_error_status(r: &ExecutionResult) -> String {
    format!("{:?}", r.promise_errors()[0].as_ref().unwrap().status())
}

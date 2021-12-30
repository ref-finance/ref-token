#![allow(unused)] 
use near_sdk_sim::{ExecutionResult, view};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
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
    pub last_proposal_id: u64,
    pub lock_amount_per_proposal: U128,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    Vote,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    WarmUp,
    InProgress,
    Approved,
    Rejected,
    Removed,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInfo{
    pub proposer: AccountId,
    pub lock_amount: U128,
    pub description: String,
    pub vote_policy: VotePolicy,
    pub kind: ProposalKind,
    pub status: ProposalStatus,
    pub vote_counts: [U128; 4],
    pub session_id: u32,
    pub start_offset: u64,
    pub lasts: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Rational {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VotePolicy {
    Relative(Rational, Rational),
    Absolute(Rational, Rational),
}

pub fn get_error_count(r: &ExecutionResult) -> u32 {
    r.promise_errors().len() as u32
}

pub fn get_error_status(r: &ExecutionResult) -> String {
    format!("{:?}", r.promise_errors()[0].as_ref().unwrap().status())
}

pub fn nano_to_sec(nano: u64) -> u32 {
    (nano / 1_000_000_000) as u32
}

pub fn sec_to_nano(sec: u32) -> u64 {
    sec as u64 * 1_000_000_000
}
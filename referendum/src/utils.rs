//! Utils stores pub info 

use std::ops::Mul;

use near_sdk::json_types::U128;
use near_sdk::{ext_contract, Gas, Timestamp, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}
use crate::proposals::VotePolicy;

/// Attach no deposit.
pub const NO_DEPOSIT: u128 = 0;

pub const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;

pub const GAS_FOR_FT_TRANSFER: Gas = 20_000_000_000_000;

/// meanwhile the max locking period
pub const MAX_SESSIONS: usize = 24;

/// each session lasts 30 days
pub const SESSION_INTERMAL: u64 = 3600 * 24 * 30 * 1_000_000_000;

/// make the default launch time to be 30 days after contract initiation
pub const DEFAULT_GENESIS_OFFSET: u64 = 3600 * 24 * 30 * 1_000_000_000;

pub const STORAGE_BALANCE_MIN_BOUND: u128 = 10_000_000_000_000_000_000_000;
/// default locking amount is 10 near for each proposal
pub const DEFAULT_LOCK_NEAR_AMOUNT_FOR_PROPOSAL: Balance = 10_000_000_000_000_000_000_000_000;

pub const DEFAULT_NONSENSE_THRESHOLD: Rational = Rational {numerator: 1, denominator: 2};

pub const DEFAULT_VP_RELATIVE: VotePolicy = VotePolicy::Relative(
    Rational {numerator: 33, denominator: 100}, 
    Rational {numerator: 1, denominator: 2}
);

pub const DEFAULT_VP_ABSOLUTE: VotePolicy = VotePolicy::Absolute(
    Rational {numerator: 45, denominator: 100}, 
    Rational {numerator: 33, denominator: 100}
);


#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct Rational {
    numerator: u32,
    denominator: u32,
}

impl From<Vec<u8>> for Rational {
    fn from(content: Vec<u8>) -> Self {
        Rational::try_from_slice(&content).unwrap()
    }
}

impl Rational {

    pub fn pass(&self, num: &Balance, denom: &Balance) -> bool {
        U256::from(*num).mul(U256::from(self.denominator)).
            ge(&U256::from(self.numerator).mul(U256::from(*denom)))
    }

    pub fn is_valid(&self) -> bool {
        self.numerator > 0 && self.denominator >= self.numerator
    }
}

pub fn nano_to_sec(nano: Timestamp) -> u32 {
    (nano / 1_000_000_000) as u32
}

pub fn sec_to_nano(sec: u32) -> Timestamp {
    sec as u64 * 1_000_000_000
}

#[ext_contract(ext_self)]
pub trait Withdraw {
    fn callback_post_withdraw(
        &mut self,
        sender_id: AccountId,
        amount: U128,
    );
}

pub fn nano_to_day(nano: Timestamp) -> u64 {
    nano / (3600 * 24 * 1_000_000_000)
}
//! Utils stores pub info 

use near_sdk::json_types::U128;
use near_sdk::{ext_contract, Gas};

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

#[ext_contract(ext_self)]
pub trait Unlock {
    fn callback_post_unlock(
        &mut self,
        sender_id: AccountId,
        amount: U128,
    );
}

use near_sdk::json_types::U128;
use near_sdk::{ext_contract, Gas, Timestamp};
use uint::construct_uint;

/// Attach no deposit.
pub const NO_DEPOSIT: u128 = 0;

pub const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;

pub const GAS_FOR_FT_TRANSFER: Gas = 20_000_000_000_000;


construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

pub fn nano_to_sec(nano: Timestamp) -> u64 {
    nano / 1_000_000_000
}


#[ext_contract(ext_self)]
pub trait XRef {
    fn callback_post_unstake(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        share: U128,
    );
}



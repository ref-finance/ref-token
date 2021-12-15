//! View functions for the contract.

use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Debug))]
pub struct ContractMetadata {
    pub version: String,
    pub owner_id: AccountId,
    pub locked_token: AccountId,
    pub undistribute_reward: U128,
    pub locked_token_amount: U128,
    pub supply: U128,
    pub prev_distribution_time: u64,
    pub reward_per_sec: U128,
}

#[near_bindgen]
impl Contract {

    /// Return contract basic info
    pub fn contract_metadata(&self) -> ContractMetadata {
        ContractMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            locked_token: self.locked_token.clone(),
            undistribute_reward: self.undistribute_reward.into(),
            locked_token_amount: self.locked_token_amount.into(),
            supply: self.ft.total_supply.into(),
            prev_distribution_time: self.prev_distribution_time,
            reward_per_sec: self.reward_per_sec.into(),
        }
    }

    // get the X-REF / REF price in decimal 8
    pub fn get_virtual_price(&self) -> U128 {
        (self.locked_token_amount * 100_000_000 / self.ft.total_supply).into()
    }
}
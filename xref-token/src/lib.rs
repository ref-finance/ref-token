/*!
* XRef NEP-141 Token contract
*
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
// Needed by `impl_fungible_token_core` for old Rust.
#[allow(unused_imports)]
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};
use crate::utils::DURATION_30DAYS_IN_SEC;
pub use crate::utils::nano_to_sec;
pub use crate::views::ContractMetadata;

mod xref;
mod utils;
mod owner;
mod views;
mod storage_impl;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub ft: FungibleToken,
    pub owner_id: AccountId,
    pub locked_token: AccountId,
    /// deposit reward that does not distribute to locked REF yet
    pub undistribute_reward: Balance,
    /// locked amount
    pub locked_token_amount: Balance,
    /// the previous distribution time in seconds
    pub prev_distribution_time_in_sec: u32,
    /// when would the reward starts to distribute
    pub reward_genesis_time_in_sec: u32,
    pub reward_per_sec: Balance,
    /// current account number in contract
    pub account_number: u64,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId, locked_token: ValidAccountId) -> Self {
        let initial_reward_genisis_time = DURATION_30DAYS_IN_SEC + nano_to_sec(env::block_timestamp());
        Contract {
            ft: FungibleToken::new(b"a".to_vec()),
            owner_id: owner_id.into(),
            locked_token: locked_token.into(),
            undistribute_reward: 0,
            locked_token_amount: 0,
            prev_distribution_time_in_sec: initial_reward_genisis_time,
            reward_genesis_time_in_sec: initial_reward_genisis_time,
            reward_per_sec: 0,
            account_number: 0,
        }
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, ft);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        let data_url = "data:image/svg+xml;base64,\
        PHN2ZyB3aWR0aD0iNTYiIGhlaWdodD0iNjIiIHZpZXdCb3g9IjAgMCA1NiA2MiIg\
        ZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4K\
        PHBhdGggZD0iTTEuODU2OTMgMTcuODg1NkMxLjg1NjkzIDE2LjAyNzggMi44ODY5\
        MiAxNC4zMjMyIDQuNTMxNTEgMTMuNDU5MkwyNS45MjE2IDIuMjIxNzJDMjcuMzc3\
        NSAxLjQ1NjgzIDI5LjExNjUgMS40NTY4MyAzMC41NzI0IDIuMjIxNzJMNTEuOTYy\
        NCAxMy40NTkyQzUzLjYwNyAxNC4zMjMyIDU0LjYzNyAxNi4wMjc4IDU0LjYzNyAx\
        Ny44ODU2VjQ1LjYzMDRDNTQuNjM3IDQ3LjYwMjEgNTMuNDc4MiA0OS4zODk4IDUx\
        LjY3ODMgNTAuMTk0N0wzMC4yODgyIDU5Ljc2MDZDMjguOTg5NCA2MC4zNDE0IDI3\
        LjUwNDYgNjAuMzQxNSAyNi4yMDU3IDU5Ljc2MDZMNC44MTU3IDUwLjE5NDdDMy4w\
        MTU3NCA0OS4zODk4IDEuODU2OTMgNDcuNjAyMSAxLjg1NjkzIDQ1LjYzMDRMMS44\
        NTY5MyAxNy44ODU2WiIgZmlsbD0idXJsKCNwYWludDBfbGluZWFyXzEyNDYxXzIw\
        NzUpIiBzdHJva2U9IiMwMEM2QTIiIHN0cm9rZS13aWR0aD0iMiIvPgo8cGF0aCBk\
        PSJNMTMuNjk3OCAyMC4zMzJMMjguMjQ3MSAxNEwyOC4yMjAyIDMwLjU0MTdMMjAu\
        MjgwMyAyMy43MTE2TDEyLjMxMzUgMzAuOTI5NVYyMi4zNjU0QzEyLjMxNTIgMjEu\
        NDkyMiAxMi44NTM4IDIwLjY5OTMgMTMuNjk3OCAyMC4zMzJaIiBmaWxsPSIjMDBD\
        NkEyIiBmaWxsLW9wYWNpdHk9IjAuNSIvPgo8cGF0aCBkPSJNMTQuMTAyMyA0Mi43\
        NjQ1TDI4LjI0NzEgNDYuODYyNkwyOC4yMjAyIDM0LjU5NDRMMjAuMjc5NCAyNy45\
        NDE0TDEyLjMxMzUgMzQuOTcyMlY0MC41Mjc0QzEyLjMxMzUgNDEuNTUxNSAxMy4w\
        MzY3IDQyLjQ1NTkgMTQuMTAyMyA0Mi43NjQ1WiIgZmlsbD0iIzAwQzZBMiIgZmls\
        bC1vcGFjaXR5PSIwLjUiLz4KPHBhdGggZD0iTTQzLjY0NDUgNDIuNzYzM0wyOC4y\
        NzQ0IDQ2Ljg2MzJMMjguMjQ2OCAzNC41MTU3TDQzLjIyMSAyMi40NjQ4QzQzLjIy\
        MSAyMi40NjQ4IDQ1Ljc5MyAyNC4zOTk2IDQ1LjAzNzMgMjcuODE5NkM0My43MDQ4\
        IDMzLjg1MTEgMzUuMTc5NiAzNS45ODY5IDM1LjE3OTYgMzUuOTg2OUw0My45MDk2\
        IDQxLjE1NjFDNDQuNjE1NSA0MS41NzggNDQuNDU1NCA0Mi41NDcgNDMuNjQ0NSA0\
        Mi43NjMzWiIgZmlsbD0iIzQ1RkZERSIvPgo8cGF0aCBkPSJNMzguNTkwMyAxOC45\
        NzkzTDI4LjI3MzQgMTRMMjguMjQ2OCAzMC40MzE1TDQwLjY5NSAyMC4zNTA5QzQw\
        LjY5NSAyMC4zNTA5IDQwLjQyNzEgMjAuMDU5NyAzOS42OTAxIDE5LjU4MDVDMzku\
        NDI4OSAxOS40MTE0IDM4LjU5MDMgMTguOTc5MyAzOC41OTAzIDE4Ljk3OTNaIiBm\
        aWxsPSIjNDVGRkRFIi8+CjxwYXRoIGQ9Ik00MC41NTEgMTYuMDEwMUw0Ni42NjAy\
        IDE4LjI4MDVDNDYuOTY3NSAxOC4zOTQzIDQ3LjE2OCAxOC42NjU2IDQ3LjE2OCAx\
        OC45NjVWMjMuMTIwN0M0Ny4xNjggMjMuNDM4OCA0Ni43MjYgMjMuNTgyMiA0Ni41\
        MDQyIDIzLjMzNDNMNDAuMjU1MyAxNi4zNjg3QzQwLjA4OTMgMTYuMTgzMSA0MC4z\
        MDYyIDE1LjkxODEgNDAuNTUxIDE2LjAxMDFaIiBmaWxsPSIjNDVGRkRFIi8+Cjxk\
        ZWZzPgo8bGluZWFyR3JhZGllbnQgaWQ9InBhaW50MF9saW5lYXJfMTI0NjFfMjA3\
        NSIgeDE9IjI4LjI0NyIgeTE9IjEiIHgyPSIyOC4yNDciIHkyPSI2MC42NzM1IiBn\
        cmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSI+CjxzdG9wIHN0b3AtY29sb3I9\
        IiMwMTEzMjAiLz4KPHN0b3Agb2Zmc2V0PSIxIiBzdG9wLWNvbG9yPSIjMDAxMzIw\
        Ii8+CjwvbGluZWFyR3JhZGllbnQ+CjwvZGVmcz4KPC9zdmc+Cg==";

        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: String::from("xRef Finance Token"),
            symbol: String::from("xREF"),
            icon: Some(String::from(data_url)),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

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
        }
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, ft);
near_contract_standards::impl_fungible_token_storage!(Contract, ft);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        let data_url = "data:image/svg+xml;base64,\
        PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPCEtLSBHZW5l\
        cmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDIxLjAuMCwgU1ZHIEV4cG9ydCBQbHVn\
        LUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPgo8c3ZnIHZlcnNp\
        b249IjEuMSIgaWQ9IkxheWVyXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8y\
        MDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxp\
        bmsiIHg9IjBweCIgeT0iMHB4IgoJIHZpZXdCb3g9IjAgMCAyODggMzI0IiBzdHls\
        ZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAyODggMzI0OyIgeG1sOnNwYWNl\
        PSJwcmVzZXJ2ZSI+CjxzdHlsZSB0eXBlPSJ0ZXh0L2NzcyI+Cgkuc3Qwe2ZpbGw6\
        IzAwQzA4Qjt9Cjwvc3R5bGU+CjxnPgoJPHBhdGggZD0iTTE3My40LDE5MS40VjI2\
        OEgyNTBMMTczLjQsMTkxLjR6IE0xMDcuMiwxMjUuMmwzMCwzMGwzMC4zLTMwLjNW\
        NjkuMmgtNjAuNFYxMjUuMnogTTEwNy4yLDE1Mi4zVjI2OGg2MC40VjE1MmwtMzAu\
        MywzMC4zCgkJTDEwNy4yLDE1Mi4zeiBNMTc3LjEsNjkuMmgtMy43VjExOUwyMTIs\
        ODAuNUMyMDEuOCw3My4yLDE4OS42LDY5LjIsMTc3LjEsNjkuMnogTTM4LDE3NS41\
        VjI2OGg2My4zVjE0Ni40bC0xNy4xLTE3LjFMMzgsMTc1LjV6CgkJIE0zOCwxNDgu\
        NWw0Ni4yLTQ2LjJsMTcuMSwxNy4xVjY5LjJIMzhWMTQ4LjV6IE0yMzYuOCwxMjgu\
        OUwyMzYuOCwxMjguOWMwLTEyLjUtMy45LTI0LjctMTEuMi0zNC44bC01Mi4xLDUy\
        djQyLjRoMy43CgkJQzIxMC4xLDE4OC41LDIzNi44LDE2MS44LDIzNi44LDEyOC45\
        eiIvPgoJPHBvbHlnb24gY2xhc3M9InN0MCIgcG9pbnRzPSIyMTAuMiw1NiAyNTAs\
        OTUuOCAyNTAsNTYgCSIvPgo8L2c+Cjwvc3ZnPgo=";

        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: String::from("X-Ref Finance Token"),
            symbol: String::from("XREF"),
            icon: Some(String::from(data_url)),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

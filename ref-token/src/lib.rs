/*!
* Ref NEP-141 Token contract
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
use near_sdk::env;
use near_sdk::{near_bindgen, log, AccountId, Balance, PanicOnDefault, PromiseOrValue};


near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub ft: FungibleToken,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner: ValidAccountId, total_supply: U128) -> Self {
        let mut contract = Contract {
            ft: FungibleToken::new(b"a".to_vec()),
        };
        let amount: Balance = total_supply.into();
        contract.ft.internal_register_account(owner.as_ref());
        contract.ft.internal_deposit(owner.as_ref() , amount);
        log!("Deposit {} token to {}", amount, owner);
        contract
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
            name: String::from("Ref Finance Token"),
            symbol: String::from("REF"),
            icon: Some(String::from(data_url)),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

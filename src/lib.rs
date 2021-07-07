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
        PHN2ZyBpZD0iTGF5ZXJfMSIgZGF0YS1uYW1lPSJMYXllciAxIiB4bWxucz0iaHR0\
        cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyODggMzI0Ij48\
        ZGVmcz48c3R5bGU+LmNscy0xe2ZpbGw6IzAwYzA4Yjt9PC9zdHlsZT48L2RlZnM+\
        PHBhdGggZD0iTTcyLDIzMS4xOWg2YTEyLjEsMTIuMSwwLDAsMSw1Ljc1LDEsNS40\
        NSw1LjQ1LDAsMCwxLDMsNSw1LjYsNS42LDAsMCwxLTQuNTYsNS42NGw2LjE3LDgu\
        ODZIODQuMDZsLTUuNzItOC40N0g3NS42djguNDdINzJabTYuNDYsOS4yNmE1Ljks\
        NS45LDAsMCwwLDMuNTYtLjgyQTIuODIsMi44MiwwLDAsMCw4MS42LDIzNWE4LjE3\
        LDguMTcsMCwwLDAtMy44Mi0uNTdINzUuNnY2LjA2WiIvPjxwYXRoIGQ9Ik05Mi4y\
        NCwyNDUuNjZjLjQzLDIuMzIsMi4yNywzLjUxLDQuNjIsMy41MWE4LjI3LDguMjcs\
        MCwwLDAsNC43NS0xLjYxdjMuMTRhMTAuMTUsMTAuMTUsMCwwLDEtNS4wOSwxLjNj\
        LTQuNDcsMC03Ljc2LTIuODktNy43Ni03LjI1YTcuMDUsNy4wNSwwLDAsMSw3LjEx\
        LTcuM2MzLjQ1LDAsNi41MSwyLjM1LDYuNTEsNi44MmE5Ljk0LDkuOTQsMCwwLDEt\
        LjA5LDEuMzlabTAtMi4zOGg2Ljk0YTMuMjYsMy4yNiwwLDAsMC0zLjM0LTMuMDZB\
        My41OSwzLjU5LDAsMCwwLDkyLjI0LDI0My4yOFoiLz48cGF0aCBkPSJNMTA2LjQs\
        MjQwLjY4aC0yLjF2LTIuOTJoMi4xdi0yLjM4YzAtMS44Ny4zNC0zLjQyLDEuMzYt\
        NC40MmE1LjE2LDUuMTYsMCwwLDEsMy41OS0xLjIxLDYuNzYsNi43NiwwLDAsMSwy\
        LjI0LjM0VjIzM2E5LjM3LDkuMzcsMCwwLDAtMS45Mi0uMjVjLTEuNTMsMC0xLjg0\
        LDEtMS44NCwyLjM1djIuNjloMy4zOXYyLjkyaC0zLjM5djExSDEwNi40WiIvPjxw\
        YXRoIGNsYXNzPSJjbHMtMSIgZD0iTTExOCwyNDguMjZhMS44NywxLjg3LDAsMSwx\
        LTEuODcsMS44N0ExLjg5LDEuODksMCwwLDEsMTE4LDI0OC4yNloiLz48cGF0aCBj\
        bGFzcz0iY2xzLTEiIGQ9Ik0xMjUuNjgsMjQwLjA1aC0yLjJ2LTIuMjloMi4yVjIz\
        NS4xYzAtMS44MS4yNi0zLjIzLDEuMjUtNC4xOWE0LjU4LDQuNTgsMCwwLDEsMy4y\
        Ni0xLjE2LDguMTksOC4xOSwwLDAsMSwyLjA5LjI4djIuMjlhOS41OCw5LjU4LDAs\
        MCwwLTItLjI1Yy0xLjc2LDAtMi4wNywxLjE5LTIuMDcsMi42NnYzaDMuNTd2Mi4y\
        OWgtMy41N3YxMS42NGgtMi41NVpNMTM2LDIzMS4xOWExLjc5LDEuNzksMCwxLDEt\
        MS43OCwxLjc4QTEuNzcsMS43NywwLDAsMSwxMzYsMjMxLjE5Wm0tMS4yNyw2LjU3\
        aDIuNTR2MTMuOTNoLTIuNTRaIi8+PHBhdGggY2xhc3M9ImNscy0xIiBkPSJNMTQx\
        Ljc0LDIzNy43NmgyLjU1djEuNzVsLjA1LDBhNyw3LDAsMCwxLDQuODctMi4wOSw0\
        LjgzLDQuODMsMCwwLDEsMy43MSwxLjUzYy43Ny44NywxLjE2LDIsMS4xNiw0LjF2\
        OC42MWgtMi41NXYtOC4xM2E0LjI4LDQuMjgsMCwwLDAtLjY1LTIuODMsMi44Niwy\
        Ljg2LDAsMCwwLTIuMjMtLjksNS44Niw1Ljg2LDAsMCwwLTQuMzYsMi4zMnY5LjU0\
        aC0yLjU1WiIvPjxwYXRoIGNsYXNzPSJjbHMtMSIgZD0iTTE1OC43OCwyNTFhMy43\
        MywzLjczLDAsMCwxLTEuNDctMy4wOSwzLjg3LDMuODcsMCwwLDEsMS44NC0zLjM5\
        LDcuMjcsNy4yNywwLDAsMSwzLjkxLS45NCwxMi4wNywxMi4wNywwLDAsMSwzLC4z\
        N1YyNDIuOGEzLDMsMCwwLDAtLjc5LTIuMjksMy43NSwzLjc1LDAsMCwwLTIuNTgt\
        Ljc3LDcuMTgsNy4xOCwwLDAsMC00LjI3LDEuMzl2LTIuNDlhOC43NCw4Ljc0LDAs\
        MCwxLDQuNjQtMS4xOSw2LjA4LDYuMDgsMCwwLDEsNC4xNiwxLjI3LDQuNzEsNC43\
        MSwwLDAsMSwxLjM5LDMuNjV2NS45MmMwLC44NS4zMSwxLjQ3LDEuMDcsMS40N2Ex\
        Ljk0LDEuOTQsMCwwLDAsLjgyLS4xOXYyLjEyYTMuNjUsMy42NSwwLDAsMS0xLjM1\
        LjI1LDMsMywwLDAsMS0yLjY5LTEuNDdoLS4wNmE2LjYsNi42LDAsMCwxLTQuMjgs\
        MS41M0E1LjU1LDUuNTUsMCwwLDEsMTU4Ljc4LDI1MVptNy4yNS0yLjQ5di0yLjYx\
        YTEwLjgzLDEwLjgzLDAsMCwwLTIuNzItLjM3Yy0xLjczLDAtMy40Mi41MS0zLjQy\
        LDIuMTYsMCwxLjM2LDEuMTYsMiwyLjY2LDJBNS44LDUuOCwwLDAsMCwxNjYsMjQ4\
        LjUyWiIvPjxwYXRoIGNsYXNzPSJjbHMtMSIgZD0iTTE3My4wNSwyMzcuNzZoMi41\
        NXYxLjc1bC4wNiwwYTYuOTEsNi45MSwwLDAsMSw0Ljg3LTIuMDksNC44LDQuOCww\
        LDAsMSwzLjcsMS41M2MuNzcuODcsMS4xNiwyLDEuMTYsNC4xdjguNjFoLTIuNTR2\
        LTguMTNhNC4zNCw0LjM0LDAsMCwwLS42NS0yLjgzLDIuOSwyLjksMCwwLDAtMi4y\
        NC0uOSw1Ljg2LDUuODYsMCwwLDAtNC4zNiwyLjMydjkuNTRoLTIuNTVaIi8+PHBh\
        dGggY2xhc3M9ImNscy0xIiBkPSJNMjAwLjMyLDI1MWE5LDksMCwwLDEtNC4xNywx\
        Yy00LjQxLDAtNy41Ni0zLjA2LTcuNTYtNy4xOSwwLTQuMywzLjQtNy4zNiw3Ljcz\
        LTcuMzZhOC4zOSw4LjM5LDAsMCwxLDMuNzcuODhWMjQxYTcuMzEsNy4zMSwwLDAs\
        MC0zLjg4LTEuMTksNC44OCw0Ljg4LDAsMCwwLTUsNSw0LjgsNC44LDAsMCwwLDUu\
        MTIsNC44Nyw2LjUyLDYuNTIsMCwwLDAsNC0xLjM2WiIvPjxwYXRoIGNsYXNzPSJj\
        bHMtMSIgZD0iTTIwNS4yNywyNDUuNDZjLjM3LDIuNzUsMi40MSw0LjI1LDUuMTUs\
        NC4yNWE4LDgsMCwwLDAsNC44Mi0xLjczdjIuNzJhOS45Myw5LjkzLDAsMCwxLTUs\
        MS4zYy00LjMsMC03LjUzLTIuODktNy41My03LjI4YTYuOTQsNi45NCwwLDAsMSw2\
        Ljk0LTcuMjdjMy40LDAsNi4zNCwyLjM4LDYuMzQsNi43NGE4LjI0LDguMjQsMCww\
        LDEtLjA5LDEuMjdabS4wNi0yaDguMjFhMy43OCwzLjc4LDAsMCwwLTMuOTQtMy43\
        MUE0LjI1LDQuMjUsMCwwLDAsMjA1LjMzLDI0My40NVoiLz48cGF0aCBkPSJNMTY0\
        LDE2NHY1Mmg1MlptLTQ1LTQ1LDIwLjQsMjAuNCwyMC42LTIwLjZWODFIMTE5Wm0w\
        LDE4LjM5VjIxNmg0MVYxMzcuMTlsLTIwLjYsMjAuNlpNMTY2LjUsODFIMTY0djMz\
        LjgxbDI2LjE2LTI2LjE3QTQwLjI5LDQwLjI5LDAsMCwwLDE2Ni41LDgxWk03Miwx\
        NTMuMTlWMjE2aDQzVjEzMy40bC0xMS42LTExLjYxWm0wLTE4LjM4LDMxLjQtMzEu\
        NEwxMTUsMTE1VjgxSDcyWk0yMDcsMTIxLjVoMGE0MC4yOSw0MC4yOSwwLDAsMC03\
        LjY0LTIzLjY2TDE2NCwxMzMuMTlWMTYyaDIuNUE0MC41LDQwLjUsMCwwLDAsMjA3\
        LDEyMS41WiIvPjxwb2x5Z29uIGNsYXNzPSJjbHMtMSIgcG9pbnRzPSIxODkgNzIg\
        MjE2IDk5IDIxNiA3MiAxODkgNzIiLz48L3N2Zz4=";

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

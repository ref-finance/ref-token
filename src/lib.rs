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
    pub fn new(owner: ValidAccountId, total_supply: Balance) -> Self {
        let mut contract = Contract {
            ft: FungibleToken::new(b"a".to_vec()),
        };
        contract.ft.internal_register_account(owner.as_ref());
        contract.ft.internal_deposit(owner.as_ref() , total_supply);
        log!("Deposit {} token to {}", total_supply, owner);
        contract
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, ft);
near_contract_standards::impl_fungible_token_storage!(Contract, ft);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: String::from("Ref Finance"),
            symbol: String::from("REF"),
            icon: Some(String::from("https://media.discordapp.net/attachments/857712764562309121/861781753596870676/reffi-stack.png")),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

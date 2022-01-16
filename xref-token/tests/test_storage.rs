use near_sdk_sim::{call, view, to_yocto};
use xref_token::ContractMetadata;

mod common;
use crate::common::init::*;

#[test]
fn test_account_number(){
    let (root, _, user, _, xref_contract) = 
        init_env(true);
    let current_xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(current_xref_info.account_number, 1);

    let user2 = root.create_user("user2".to_string(), to_yocto("100"));
    call!(user2, xref_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    let current_xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(current_xref_info.account_number, 2);

    call!(user2, xref_contract.storage_unregister(None), deposit = 1).assert_success();
    let current_xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(current_xref_info.account_number, 1);

    call!(user, xref_contract.storage_unregister(None), deposit = 1).assert_success();
    let current_xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(current_xref_info.account_number, 0);
}
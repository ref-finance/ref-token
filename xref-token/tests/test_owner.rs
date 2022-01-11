use near_sdk_sim::{call, view, to_yocto};
use xref_token::ContractMetadata;
use near_sdk::json_types::U128;

mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_reset_reward_genesis_time(){
    let (root, owner, _, _, xref_contract) = 
        init_env(true);
    
    let current_timestamp = root.borrow_runtime().cur_block.block_timestamp;
    call!(
        owner,
        xref_contract.reset_reward_genesis_time_in_sec(nano_to_sec(current_timestamp) + 10)
    ).assert_success();

    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_genesis_time_in_sec, nano_to_sec(current_timestamp) + 10);
}

#[test]
fn test_reset_reward_genesis_time_use_past_time(){
    let (root, owner, _, _, xref_contract) = 
        init_env(true);
    
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();

    let current_timestamp = root.borrow_runtime().cur_block.block_timestamp;
    let out_come = call!(
        owner,
        xref_contract.reset_reward_genesis_time_in_sec(nano_to_sec(current_timestamp) - 1)
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_RESET_TIME_IS_PAST_TIME"));

    let xref_info1 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_genesis_time_in_sec, xref_info1.reward_genesis_time_in_sec);
}

#[test]
fn test_reward_genesis_time_passed(){
    let (root, owner, _, _, xref_contract) = 
        init_env(true);
    
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();

    root.borrow_runtime_mut().cur_block.block_timestamp = (xref_info.reward_genesis_time_in_sec + 1) as u64 * 1_000_000_000;
    let current_timestamp = root.borrow_runtime().cur_block.block_timestamp;
    let out_come = call!(
        owner,
        xref_contract.reset_reward_genesis_time_in_sec(nano_to_sec(current_timestamp) + 1)
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_REWARD_GENESIS_TIME_PASSED"));

    let xref_info1 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_genesis_time_in_sec, xref_info1.reward_genesis_time_in_sec);
}

#[test]
fn test_modify_reward_per_sec(){
    let (_, owner, _, _, xref_contract) = 
        init_env(true);
    
    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("1").into(), true)
    )
    .assert_success();
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_per_sec.0, to_yocto("1"));
}
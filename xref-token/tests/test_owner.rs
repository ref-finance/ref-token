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
    let (root, owner, _, ref_contract, xref_contract) = 
        init_env(true);
    
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let init_genesis_time = xref_info.reward_genesis_time_in_sec;
    assert_eq!(init_genesis_time, xref_info.prev_distribution_time_in_sec);

    // reward_distribute won't touch anything before genesis time
    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("1").into(), true)
    )
    .assert_success();
    call!(
        owner,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("100").into(), None, "reward".to_string()),
        deposit = 1
    )
    .assert_success();
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(init_genesis_time, xref_info.reward_genesis_time_in_sec);
    assert_eq!(init_genesis_time, xref_info.prev_distribution_time_in_sec);
    assert_eq!(U128(to_yocto("100")), xref_info.undistribute_reward);
    assert_eq!(U128(to_yocto("1")), xref_info.reward_per_sec);
    assert_eq!(U128(to_yocto("100")), xref_info.cur_undistribute_reward);
    assert_eq!(U128(to_yocto("0")), xref_info.cur_locked_token_amount);

    // and reward won't be distributed before genesis time
    root.borrow_runtime_mut().cur_block.block_timestamp = 100_000_000_000;
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(U128(to_yocto("100")), xref_info.cur_undistribute_reward);
    assert_eq!(U128(to_yocto("0")), xref_info.cur_locked_token_amount);

    // and nothing happen even if some action invoke the reward distribution before genesis time
    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("0.5").into(), true)
    )
    .assert_success();
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(init_genesis_time, xref_info.reward_genesis_time_in_sec);
    assert_eq!(init_genesis_time, xref_info.prev_distribution_time_in_sec);
    assert_eq!(U128(to_yocto("100")), xref_info.undistribute_reward);
    assert_eq!(U128(to_yocto("0.5")), xref_info.reward_per_sec);
    assert_eq!(U128(to_yocto("100")), xref_info.cur_undistribute_reward);
    assert_eq!(U128(to_yocto("0")), xref_info.cur_locked_token_amount);
    
    // change genesis time would also change prev_distribution_time_in_sec
    let current_timestamp = root.borrow_runtime().cur_block.block_timestamp;
    call!(
        owner,
        xref_contract.reset_reward_genesis_time_in_sec(nano_to_sec(current_timestamp) + 50)
    ).assert_success();
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_genesis_time_in_sec, nano_to_sec(current_timestamp) + 50);
    assert_eq!(xref_info.prev_distribution_time_in_sec, nano_to_sec(current_timestamp) + 50);
    assert_eq!(U128(to_yocto("100")), xref_info.undistribute_reward);
    assert_eq!(U128(to_yocto("0.5")), xref_info.reward_per_sec);
    assert_eq!(U128(to_yocto("100")), xref_info.cur_undistribute_reward);
    assert_eq!(U128(to_yocto("0")), xref_info.cur_locked_token_amount);

    // when it past genesis time
    root.borrow_runtime_mut().cur_block.block_timestamp = current_timestamp + 60_000_000_000;
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(U128(to_yocto("95")), xref_info.cur_undistribute_reward);
    assert_eq!(U128(to_yocto("5")), xref_info.cur_locked_token_amount);
    // when some call invoke reward distribution after reward genesis time
    root.borrow_runtime_mut().cur_block.block_timestamp = current_timestamp + 70_000_000_000;
    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("1").into(), true)
    )
    .assert_success();
    root.borrow_runtime_mut().cur_block.block_timestamp = current_timestamp + 80_000_000_000;
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_genesis_time_in_sec, nano_to_sec(current_timestamp) + 50);
    assert_eq!(xref_info.prev_distribution_time_in_sec, nano_to_sec(current_timestamp) + 71);
    assert_eq!(U128(to_yocto("89.5")), xref_info.undistribute_reward);
    assert_eq!(U128(to_yocto("10.5")), xref_info.locked_token_amount);
    assert_eq!(U128(to_yocto("1")), xref_info.reward_per_sec);
    assert_eq!(U128(to_yocto("80.5")), xref_info.cur_undistribute_reward);
    assert_eq!(U128(to_yocto("19.5")), xref_info.cur_locked_token_amount);
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
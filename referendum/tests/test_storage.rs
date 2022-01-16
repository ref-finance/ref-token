use near_sdk_sim::{call, view, to_yocto};
mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_storage_deposit_not_launched(){
    let (_, _, user, _, referendum_contract) = 
        init_env(false);

    let out_come = call!(user, referendum_contract.storage_deposit(None, None), deposit = to_yocto("1"));
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_LAUNCHED"));
}

#[test]
fn test_storage_deposit_normal(){
    let (root, owner, user, _, referendum_contract) = 
        init_env(false);

    let current_timestamp = root.borrow_runtime().current_block().block_timestamp;
    call!(
        owner,
        referendum_contract.modify_genesis_timestamp(nano_to_sec(current_timestamp) + 10)
    )
    .assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = sec_to_nano(nano_to_sec(current_timestamp) + 10);

    let orig_user_balance = user.account().unwrap().amount;
    call!(user, referendum_contract.storage_deposit(None, None), deposit = to_yocto("0.01")).assert_success();
    assert!(orig_user_balance - user.account().unwrap().amount > to_yocto("0.01"));
    assert!(orig_user_balance - user.account().unwrap().amount < to_yocto("0.011"));
}

#[test]
fn test_storage_deposit_repeat(){
    let (root, owner, user, _, referendum_contract) = 
        init_env(false);

    let current_timestamp = root.borrow_runtime().current_block().block_timestamp;
    call!(
        owner,
        referendum_contract.modify_genesis_timestamp(nano_to_sec(current_timestamp) + 10)
    )
    .assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = sec_to_nano(nano_to_sec(current_timestamp) + 10);

    let orig_user_balance = user.account().unwrap().amount;
    call!(user, referendum_contract.storage_deposit(None, None), deposit = to_yocto("0.01")).assert_success();
    assert!(orig_user_balance - user.account().unwrap().amount > to_yocto("0.01"));
    assert!(orig_user_balance - user.account().unwrap().amount < to_yocto("0.011"));

    let orig_user_balance = user.account().unwrap().amount;
    call!(user, referendum_contract.storage_deposit(None, None), deposit = to_yocto("0.01")).assert_success();
    assert!(orig_user_balance - user.account().unwrap().amount < to_yocto("0.001"));
}

#[test]
fn test_storage_deposit_refund(){
    let (root, owner, user, _, referendum_contract) = 
        init_env(false);
    
    let current_timestamp = root.borrow_runtime().current_block().block_timestamp;
    call!(
        owner,
        referendum_contract.modify_genesis_timestamp(nano_to_sec(current_timestamp) + 10)
    )
    .assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = sec_to_nano(nano_to_sec(current_timestamp) + 10);

    let orig_user_balance = user.account().unwrap().amount;
    call!(user, referendum_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    assert!(orig_user_balance - user.account().unwrap().amount > to_yocto("0.01"));
    assert!(orig_user_balance - user.account().unwrap().amount < to_yocto("0.011"));
}

#[test]
fn test_storage_unregister_normal(){
    let (root, _, user, xref_contract, referendum_contract) = 
        init_env(true);
    
    let orig_user_balance = user.account().unwrap().amount;
    call!(user, referendum_contract.storage_unregister(None), deposit = 1).assert_success();
    assert!(user.account().unwrap().amount - orig_user_balance > to_yocto("0.009"));
    assert!(user.account().unwrap().amount - orig_user_balance < to_yocto("0.01"));

    call!(user, referendum_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = sec_to_nano(view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp_sec) + 31 * 3600 * 24 * 1_000_000_000;

    call!(
        user,
        referendum_contract.withdraw(),
        deposit = 1
    ).assert_success();

    call!(user, referendum_contract.storage_unregister(None), deposit = 1).assert_success();
}

#[test]
fn test_storage_unregister_repeat(){
    let (_, _, user, _, referendum_contract) = 
        init_env(true);

    let orig_user_balance = user.account().unwrap().amount;
    call!(user, referendum_contract.storage_unregister(None), deposit = 1).assert_success();
    assert!(user.account().unwrap().amount - orig_user_balance > to_yocto("0.009"));
    assert!(user.account().unwrap().amount - orig_user_balance < to_yocto("0.01"));

    let orig_user_balance = user.account().unwrap().amount;
    call!(user, referendum_contract.storage_unregister(None), deposit = 1).assert_success();
    assert!(orig_user_balance - user.account().unwrap().amount < to_yocto("0.001"));
}

#[test]
fn test_storage_unregister_before_unlock(){
    let (_, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let out_come = call!(user, referendum_contract.storage_unregister(None), deposit = 1);
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_NOT_UNLOCK"));
}

#[test]
fn storage_withdraw(){
    let (_, _, user, _, referendum_contract) = 
        init_env(true);

    let out_come = call!(user, referendum_contract.storage_withdraw(None), deposit = 1);
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NO_STORAGE_CAN_WITHDRAW"));
}
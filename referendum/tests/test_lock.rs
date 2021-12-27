use near_sdk_sim::{call, view, to_yocto};
mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_lock_user_not_register(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let user = root.create_user("user_not_register".to_string(), to_yocto("100"));
    call!(user, xref_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(root, xref_contract.mint(user.valid_account_id(), to_yocto("100").into())).assert_success();

    let out_come = call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "10".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_USER_NOT_REGISTER"));
}

#[test]
fn test_lock_new_lasts1(){
    let (_, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_session, 0);
    assert_eq!(contract_metadata.cur_total_ballot.0, to_yocto("10"));

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, to_yocto("10"));
    assert_eq!(account_info.ballot_amount.0, to_yocto("10"));
    assert_eq!(account_info.unlocking_session_id, 0);

    let session_state = view!(referendum_contract.get_session_state(0)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 0);
    assert_eq!(session_state.expire_amount.0, to_yocto("10"));
}

#[test]
fn test_lock_new_lasts10(){
    let (_, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "10".to_string()),
        deposit = 1
    ).assert_success();

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_session, 0);
    assert_eq!(contract_metadata.cur_total_ballot.0, to_yocto("10") * 10);

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_session, 0);
    assert_eq!(contract_metadata.cur_total_ballot.0, to_yocto("10") * 10);

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, to_yocto("10"));
    assert_eq!(account_info.ballot_amount.0, to_yocto("10") * 10);
    assert_eq!(account_info.unlocking_session_id, 9);

    let session_state = view!(referendum_contract.get_session_state(9)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 9);
    assert_eq!(session_state.expire_amount.0, to_yocto("10") * 10);
}

#[test]
fn test_lock_new_when_session_last_day(){
    let (root, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 29 * 3600 * 24 * 1_000_000_000;

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_session, 0);
    assert_eq!(contract_metadata.cur_total_ballot.0, to_yocto("1"));

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, to_yocto("30"));
    assert_eq!(account_info.ballot_amount.0, to_yocto("1"));
    assert_eq!(account_info.unlocking_session_id, 0);

    let session_state = view!(referendum_contract.get_session_state(0)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 0);
    assert_eq!(session_state.expire_amount.0, to_yocto("1"));
}

#[test]
fn test_lock_new_expire(){
    let (root, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, to_yocto("10"));
    assert_eq!(account_info.ballot_amount.0, to_yocto("10"));
    assert_eq!(account_info.unlocking_session_id, 0);

    let session_state = view!(referendum_contract.get_session_state(0)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 0);
    assert_eq!(session_state.expire_amount.0, to_yocto("10"));

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 31 * 3600 * 24 * 1_000_000_000;

    let out_come = call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "1".to_string()),
        deposit = 1
    );

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_HAS_VALID_RUNNING_LOCKING_OR_BALLOTS_EXPIRE_NOT_UNLOCK"));
}

#[test]
fn test_lock_append_expire(){
    let (root, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, to_yocto("10"));
    assert_eq!(account_info.ballot_amount.0, to_yocto("10"));
    assert_eq!(account_info.unlocking_session_id, 0);

    let session_state = view!(referendum_contract.get_session_state(0)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 0);
    assert_eq!(session_state.expire_amount.0, to_yocto("10"));

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 31 * 3600 * 24 * 1_000_000_000;

    let out_come = call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    );

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_BALLOTS_EXPIRE_NOT_UNLOCK"));
}

#[test]
fn test_lock_append(){
    let (_, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "10".to_string()),
        deposit = 1
    ).assert_success();

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    ).assert_success();

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_session, 0);
    assert_eq!(contract_metadata.cur_total_ballot.0, to_yocto("20") * 10);

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, to_yocto("20"));
    assert_eq!(account_info.ballot_amount.0, to_yocto("20") * 10);
    assert_eq!(account_info.unlocking_session_id, 9);

    let session_state = view!(referendum_contract.get_session_state(9)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 9);
    assert_eq!(session_state.expire_amount.0, to_yocto("20") * 10);
}

#[test]
fn test_lock_append_add_msg(){
    let (_, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "10".to_string()),
        deposit = 1
    ).assert_success();

    let out_come = call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "10".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_HAS_VALID_RUNNING_LOCKING"));
}

#[test]
fn test_lock_append_when_no_lock(){
    let (_, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    let out_come = call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    );

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_HAS_NO_RUNNING_LOCKING"));
}

#[test]
fn test_lock_append_illegal_msg(){
    let (_, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    let out_come = call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("10").into(), None, "0".to_string()),
        deposit = 1
    );

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ILLEGAL_MSG"));
}

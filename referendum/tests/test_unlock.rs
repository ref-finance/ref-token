use near_sdk_sim::{call, view, to_yocto};
use near_sdk::json_types::U128;
mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_unlock(){
    let (root, _, user, xref_contract, referendum_contract) = 
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

    assert_eq!(view!(xref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0, to_yocto("90"));

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp  + 31 * 3600 * 24 * 1_000_000_000;

    call!(
        user,
        referendum_contract.unlock(),
        deposit = 1
    ).assert_success();

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_session, 1);
    assert_eq!(contract_metadata.cur_total_ballot.0, 0);

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, 0);
    assert_eq!(account_info.ballot_amount.0, 0);
    assert_eq!(account_info.unlocking_session_id, 0);

    let session_state = view!(referendum_contract.get_session_state(0)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 24);
    assert_eq!(session_state.expire_amount.0, 0);

    assert_eq!(view!(xref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0, to_yocto("100"));

    call!(
        user,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_session, 1);
    assert_eq!(contract_metadata.cur_total_ballot.0, to_yocto("29"));

    let account_info = view!(referendum_contract.get_user_base_info(user.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(account_info.locking_amount.0, to_yocto("30"));
    assert_eq!(account_info.ballot_amount.0, to_yocto("29"));
    assert_eq!(account_info.unlocking_session_id, 1);

    let session_state = view!(referendum_contract.get_session_state(1)).unwrap_json::<SessionState>();
    assert_eq!(session_state.session_id, 1);
    assert_eq!(session_state.expire_amount.0, to_yocto("29"));

    assert_eq!(view!(xref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0, to_yocto("70"));
}

#[test]
fn test_unlock_ahead(){
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

    assert_eq!(view!(xref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0, to_yocto("90"));

    call!(
        user,
        referendum_contract.unlock(),
        deposit = 1
    ).assert_success();

    assert_eq!(view!(xref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0, to_yocto("90"));
}
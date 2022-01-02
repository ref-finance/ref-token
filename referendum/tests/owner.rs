use near_sdk_sim::{call, view, to_yocto};
use near_sdk::json_types::U128;
mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_owner(){
    let (root, owner, user, _, referendum_contract) = 
        init_env(false);
    
    call!(
        owner,
        referendum_contract.set_owner(user.valid_account_id())
    ).assert_success();

    let out_come = call!(
        owner,
        referendum_contract.set_owner(user.valid_account_id())
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ALLOWED"));

    let current_timestamp = root.borrow_runtime().current_block().block_timestamp;
    call!(
        user,
        referendum_contract.modify_genesis_timestamp(nano_to_sec(current_timestamp) + 10)
    ).assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = sec_to_nano(nano_to_sec(current_timestamp) + 10);

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.genesis_timestamp_sec, nano_to_sec(current_timestamp) + 10);

    assert_eq!(contract_metadata.lock_amount_per_proposal.0, to_yocto("10"));
    call!(
        user,
        referendum_contract.modify_endorsement_amount(U128(to_yocto("20")))
    ).assert_success();
    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.lock_amount_per_proposal.0, to_yocto("20"));
}
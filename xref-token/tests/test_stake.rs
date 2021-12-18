use near_sdk_sim::{call, view, to_yocto};
use xref_token::ContractMetadata;
use near_sdk::json_types::U128;

mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_stake(){
    let (_, _, user, ref_contract, xref_contract) = 
        init_env(true);

    call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();

    let current_xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&current_xref_info, 0, to_yocto("10"), to_yocto("10"));
    assert_eq!(100000000_u128, view!(xref_contract.get_virtual_price()).unwrap_json::<U128>().0);
    assert_eq!(to_yocto("90"), view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);
}

#[test]
fn test_stake_no_register(){
    let (_, _, user, ref_contract, xref_contract) = 
    init_env(false);
    
    let out_come = call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("Account not registered."));

    assert_eq!(to_yocto("100"), view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    let current_xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&current_xref_info, 0, 0, 0);
}

#[test]
fn test_stake_zero(){
    let (_, _, user, ref_contract, xref_contract) = 
    init_env(true);
    
    let out_come = call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("0").into(), None, "".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("The amount should be a positive number"));

    assert_eq!(to_yocto("100"), view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    let current_xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&current_xref_info, 0, 0, 0);
}
use near_sdk_sim::{call, view, to_yocto};
use xref_token::ContractMetadata;
use near_sdk::json_types::U128;

mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_reward(){
    let (root, owner, user, ref_contract, xref_contract) = 
        init_env(true);
    let mut total_reward = 0;
    let mut total_locked = 0;
    let mut total_supply = 0;

    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("1").into(), true)
    )
    .assert_success();
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_per_sec.0, to_yocto("1"));

    let current_timestamp = root.borrow_runtime_mut().cur_block.block_timestamp;
    let out_come = call!(
        owner,
        xref_contract.reset_reward_genesis_time_in_sec(nano_to_sec(current_timestamp) + 10)
    );
    assert!(out_come.unwrap_json::<bool>());
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_genesis_time_in_sec, nano_to_sec(current_timestamp) + 10);
    
    root.borrow_runtime_mut().cur_block.block_timestamp = (nano_to_sec(current_timestamp) + 10) as u64 * 1_000_000_000;

    //add reward trigger distribute_reward, just update prev_distribution_time
    call!(
        owner,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("100").into(), None, "reward".to_string()),
        deposit = 1
    )
    .assert_success();
    total_reward += to_yocto("100");

    let xref_info0 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&xref_info0, to_yocto("100"), 0, 0);
    assert_eq!(to_yocto("1"), xref_info0.reward_per_sec.0);

    
    //stake trigger distribute_reward
    call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    total_locked += to_yocto("10");
    total_supply += to_yocto("10");

    let xref_info1 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let time_diff = xref_info1.prev_distribution_time_in_sec - xref_info0.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info1.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info1.reward_per_sec.0;
    assert_xref(&xref_info1, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("90"), view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    assert!(root.borrow_runtime_mut().produce_block().is_ok());

    //modify_reward_per_sec trigger distribute_reward
    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("1").into(), true)
    )
    .assert_success();
    let xref_info2 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info2.reward_per_sec.0, to_yocto("1"));

    let time_diff = xref_info2.prev_distribution_time_in_sec - xref_info1.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info2.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info2.reward_per_sec.0;
    assert_xref(&xref_info2, total_reward, total_locked, total_supply);
    
    assert!(root.borrow_runtime_mut().produce_block().is_ok());

    //modify_reward_per_sec not trigger distribute_reward
    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("1").into(), false)
    )
    .assert_success();
    let xref_info2_1 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();

    let time_diff = xref_info2_1.prev_distribution_time_in_sec - xref_info2.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info2_1.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info2_1.reward_per_sec.0;
    assert_xref(&xref_info2_1, total_reward, total_locked, total_supply);
    assert_eq!(time_diff, 0);
    
    assert!(root.borrow_runtime_mut().produce_block().is_ok());

    //nothing trigger distribute_reward
    let xref_info3 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&xref_info3, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("90"), view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);
    
    //add reward trigger distribute_reward
    call!(
        owner,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("100").into(), None, "reward".to_string()),
        deposit = 1
    )
    .assert_success();
    total_reward += to_yocto("100");

    let xref_info4 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let time_diff = xref_info4.prev_distribution_time_in_sec - xref_info3.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info4.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info4.reward_per_sec.0;
    assert_xref(&xref_info4, total_reward, total_locked, total_supply);

    assert!(root.borrow_runtime_mut().produce_block().is_ok());

    //unstake trigger distribute_reward
    call!(
        user,
        xref_contract.unstake(to_yocto("10").into()),
        deposit = 1
    )
    .assert_success();

    let xref_info5 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let time_diff = xref_info5.prev_distribution_time_in_sec - xref_info4.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info5.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info5.reward_per_sec.0;

    let unlocked = (U256::from(to_yocto("10")) * U256::from(total_locked) / U256::from(total_supply)).as_u128();
    total_locked -= unlocked;
    total_supply -= to_yocto("10");

    assert_eq!(0, total_locked);
    assert_eq!(0, total_supply);
    assert_xref(&xref_info5, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("90") + unlocked, view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    assert!(root.borrow_runtime_mut().produce_blocks(1000).is_ok());

    //nothing trigger distribute_reward
    let xref_info6 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&xref_info6, total_reward, total_locked, total_supply);

    //stake trigger distribute_reward，total_reward less then distribute_reward
    call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    
    total_locked += to_yocto("10");
    total_supply += to_yocto("10");

    let xref_info7 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let time_diff = xref_info7.prev_distribution_time_in_sec - xref_info6.prev_distribution_time_in_sec;
    assert!(total_reward < time_diff as u128 * xref_info7.reward_per_sec.0);
    total_locked += total_reward;
    total_reward -= total_reward;
    
    assert_xref(&xref_info7, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("80") + unlocked, view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    //stake when total_locked contains reward
    call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();

    total_supply += (U256::from(to_yocto("10")) * U256::from(total_supply) / U256::from(total_locked)).as_u128();
    total_locked += to_yocto("10");

    let xref_info8 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&xref_info8, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("70") + unlocked, view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);
}

#[test]
fn test_no_reward_befroe_reset_reward_genesis_time(){
    let (root, owner, user, ref_contract, xref_contract) = 
        init_env(true);
    let mut total_reward = 0;
    let mut total_locked = 0;
    let mut total_supply = 0;

    call!(
        owner,
        xref_contract.modify_reward_per_sec(to_yocto("1").into(), true)
    )
    .assert_success();
    let xref_info = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(xref_info.reward_per_sec.0, to_yocto("1"));

    //add reward trigger distribute_reward, just update prev_distribution_time
    call!(
        owner,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("100").into(), None, "reward".to_string()),
        deposit = 1
    )
    .assert_success();
    total_reward += to_yocto("100");

    let xref_info1 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_xref(&xref_info1, to_yocto("100"), 0, 0);
    assert_eq!(to_yocto("1"), xref_info1.reward_per_sec.0);

    //stake trigger distribute_reward
    call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    total_locked += to_yocto("10");
    total_supply += to_yocto("10");

    let xref_info2 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let time_diff = xref_info2.prev_distribution_time_in_sec - xref_info1.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info2.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info2.reward_per_sec.0;
    assert_xref(&xref_info2, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("90"), view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    assert!(root.borrow_runtime_mut().produce_blocks(10).is_ok());

    //stake trigger distribute_reward again
    call!(
        user,
        ref_contract.ft_transfer_call(xref_contract.valid_account_id(), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    total_locked += to_yocto("10");
    total_supply += to_yocto("10");

    let xref_info3 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let time_diff = xref_info3.prev_distribution_time_in_sec - xref_info2.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info3.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info3.reward_per_sec.0;
    assert_xref(&xref_info3, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("80"), view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    assert_eq!(xref_info3.undistribute_reward.0, to_yocto("100"));
    assert_eq!(xref_info3.locked_token_amount.0, to_yocto("20"));

    assert!(root.borrow_runtime_mut().produce_blocks(10).is_ok());

    //unstake trigger distribute_reward
    call!(
        user,
        xref_contract.unstake(to_yocto("10").into()),
        deposit = 1
    )
    .assert_success();

    let xref_info4 = view!(xref_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    let time_diff = xref_info4.prev_distribution_time_in_sec - xref_info3.prev_distribution_time_in_sec;
    total_reward -= time_diff as u128 * xref_info4.reward_per_sec.0;
    total_locked += time_diff as u128 * xref_info4.reward_per_sec.0;

    let unlocked = (U256::from(to_yocto("10")) * U256::from(total_locked) / U256::from(total_supply)).as_u128();
    total_locked -= unlocked;
    total_supply -= to_yocto("10");

    assert_eq!(to_yocto("10"), total_locked);
    assert_eq!(to_yocto("10"), total_supply);
    assert_xref(&xref_info4, total_reward, total_locked, total_supply);
    assert_eq!(to_yocto("80") + unlocked, view!(ref_contract.ft_balance_of(user.valid_account_id())).unwrap_json::<U128>().0);

    assert_eq!(unlocked, to_yocto("10"));
    assert_eq!(xref_info4.undistribute_reward.0, to_yocto("100"));
    assert_eq!(xref_info4.locked_token_amount.0, to_yocto("10"));
}
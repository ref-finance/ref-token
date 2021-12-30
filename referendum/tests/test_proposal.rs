use near_sdk_sim::{call, view, to_yocto};
use near_sdk::json_types::U128;
use near_sdk::borsh::BorshSerialize;
mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_add_proposal(){
    let (root, owner, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let vote_policy = VotePolicy::Relative(Rational{numerator:1, denominator:2}, Rational{numerator:1, denominator:2});
    
    call!(
        owner,
        referendum_contract.modify_vote_policy(vote_policy.try_to_vec().unwrap().into())
    ).assert_success();

    let orig_user_balance = proposal_user.account().unwrap().amount;
    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 1, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    );

    assert_eq!(out_come.unwrap_json::<u64>(), 0);
    assert!(orig_user_balance - proposal_user.account().unwrap().amount > to_yocto("10"));
    assert!(orig_user_balance - proposal_user.account().unwrap().amount < to_yocto("10.11"));

    let proposal_info = view!(referendum_contract.get_proposal_info(0)).unwrap_json::<ProposalInfo>();

    assert_eq!(proposal_info.proposer, proposal_user.account_id);
    assert_eq!(proposal_info.lock_amount.0, 10_000_000_000_000_000_000_000_000);
    assert_eq!(proposal_info.description, "test proposal".to_string());
    assert_eq!(proposal_info.vote_policy, vote_policy);
    assert_eq!(proposal_info.kind, ProposalKind::Vote);
    assert_eq!(proposal_info.status, ProposalStatus::WarmUp);
    assert_eq!(proposal_info.vote_counts, [U128(0); 4]);
    assert_eq!(proposal_info.session_id, 1);
    assert_eq!(proposal_info.start_offset, sec_to_nano(1000));
    assert_eq!(proposal_info.lasts, sec_to_nano(100000));

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.last_proposal_id, 1);

    assert_eq!(view!(referendum_contract.get_proposal_ids_in_session(1)).unwrap_json::<Vec<u64>>(), [0]);
}

#[test]
fn test_add_proposal_not_enough_lock_near(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 1, 1000, 100000),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ENOUGH_LOCK_NEAR"));
}

#[test]
fn test_add_proposal_refund(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let orig_user_balance = proposal_user.account().unwrap().amount;
    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 1, 1000, 100000),
        deposit = 20_000_000_000_000_000_000_000_000
    ).assert_success();
    assert!(orig_user_balance - proposal_user.account().unwrap().amount > to_yocto("10"));
    assert!(orig_user_balance - proposal_user.account().unwrap().amount < to_yocto("10.11"));
}

#[test]
fn test_add_proposal_start_time_lt_current_time(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 1, 7 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_PROPOSAL_START_TIME_NEED_GE_CURRENT_TIME"));
}

#[test]
fn test_add_proposal_end_time_gt_next_session_begin_time(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 60 * 60, 30 * 24 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(get_error_count(&out_come), 1);
    println!("{}", get_error_status(&out_come));
    assert!(get_error_status(&out_come).contains("ERR_PROPOSAL_END_TIME_NEED_LE_NEXT_SESSION_BEGIN_TIME"));
}

#[test]
fn test_add_proposal_session_id_before_current_session(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 31 * 3600 * 24 * 1_000_000_000;

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 60 * 60, 24 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_SESSION_ID_NEED_GE_CURRENT_SESSION_ID"));
}

#[test]
fn test_remove_proposal_during_warm_up(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 1, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    let orig_user_balance = proposal_user.account().unwrap().amount;
    assert!(call!(
        proposal_user,
        referendum_contract.remove_proposal(0)
    ).unwrap_json::<bool>());
    assert!(proposal_user.account().unwrap().amount - orig_user_balance > to_yocto("9.99"));
    assert!(proposal_user.account().unwrap().amount - orig_user_balance < to_yocto("10"));

    assert!(format!("{}", view!(referendum_contract.get_proposal_info(0)).unwrap_err()).contains("Err_INVALID_PROPOSAL_IDX"));
}

#[test]
fn test_remove_proposal_during_in_progress(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 60 * 60, 7 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp += 3600 * 24 * 1_000_000_000;

    let out_come = call!(
        proposal_user,
        referendum_contract.remove_proposal(0)
    );
    assert!(!out_come.unwrap_json::<bool>());
}

#[test]
fn test_remove_proposal_no_proposal(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 60 * 60, 7 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    let out_come = call!(
        proposal_user,
        referendum_contract.remove_proposal(1)
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NO_PROPOSAL"));
}

#[test]
fn test_remove_proposal_not_allow(){
    let (root, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 60 * 60, 7 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    let out_come = call!(
        user,
        referendum_contract.remove_proposal(0)
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ALLOW"));
}

#[test]
fn test_redeem(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    let out_come = call!(
        proposal_user,
        referendum_contract.redeem_near_in_expired_proposal(0)
    );
    
    assert!(!out_come.unwrap_json::<bool>());

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 31 * 3600 * 24 * 1_000_000_000;

    let orig_user_balance = proposal_user.account().unwrap().amount;
    let out_come = call!(
        proposal_user,
        referendum_contract.redeem_near_in_expired_proposal(0)
    );

    assert!(out_come.unwrap_json::<bool>());
    assert!(proposal_user.account().unwrap().amount - orig_user_balance > to_yocto("9.99"));
    assert!(proposal_user.account().unwrap().amount - orig_user_balance < to_yocto("10"));

    let proposal_info = view!(referendum_contract.get_proposal_info(0)).unwrap_json::<ProposalInfo>();
    assert_eq!(proposal_info.status, ProposalStatus::Expired);
    assert_eq!(proposal_info.lock_amount.0, 0);
}

#[test]
fn test_redeem_no_proposal(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 31 * 3600 * 24 * 1_000_000_000;

    let out_come = call!(
        proposal_user,
        referendum_contract.redeem_near_in_expired_proposal(1)
    );

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NO_PROPOSAL"));
}

#[test]
fn test_redeem_not_allow(){
    let (root, _, user, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 31 * 3600 * 24 * 1_000_000_000;

    let out_come = call!(
        user,
        referendum_contract.redeem_near_in_expired_proposal(0)
    );

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ALLOW"));
}

#[test]
fn test_proposal_ids_in_session(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, _, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 1, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(out_come.unwrap_json::<u64>(), 0);
    
    assert_eq!(view!(referendum_contract.get_proposal_ids_in_session(1)).unwrap_json::<Vec<u64>>(), [0]);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 0, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(out_come.unwrap_json::<u64>(), 1);
    
    assert_eq!(view!(referendum_contract.get_proposal_ids_in_session(0)).unwrap_json::<Vec<u64>>(), [1]);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 1, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(out_come.unwrap_json::<u64>(), 2);
    
    assert_eq!(view!(referendum_contract.get_proposal_ids_in_session(1)).unwrap_json::<Vec<u64>>(), [0,2]);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), 0.into(), 10, 1000, 100000),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(out_come.unwrap_json::<u64>(), 3);
    
    assert_eq!(view!(referendum_contract.get_proposal_ids_in_session(10)).unwrap_json::<Vec<u64>>(), [3]);

    assert!(call!(
        proposal_user,
        referendum_contract.remove_proposal(2)
    ).unwrap_json::<bool>());

    assert_eq!(view!(referendum_contract.get_proposal_ids_in_session(1)).unwrap_json::<Vec<u64>>(), [0]);

}
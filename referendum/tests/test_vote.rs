use near_sdk_sim::{call, view, to_yocto};
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::U128;
mod common;
use crate::common::{
    init::*,
    utils::*
};

#[test]
fn test_vote(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, vote_user1, vote_user2) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let vote_policy1 = VotePolicy::Relative(Rational{numerator:1, denominator:2}, Rational{numerator:1, denominator:2});
    let vote_policy2 = VotePolicy::Absolute(Rational{numerator:1, denominator:2}, Rational{numerator:2, denominator:2});

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal1".to_string(), "vote".into(), vote_policy1.try_to_vec().unwrap().into(), 0, 24 * 60 * 60, 24 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(out_come.unwrap_json::<u64>(), 0);

    let out_come = call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal2".to_string(), "vote".into(), vote_policy2.try_to_vec().unwrap().into(), 0, 24 * 60 * 60, 24 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    );
    assert_eq!(out_come.unwrap_json::<u64>(), 1);

    call!(
        vote_user1,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    call!(
        vote_user2,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let contract_metadata = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>();
    assert_eq!(contract_metadata.cur_total_ballot.0, to_yocto("60"));

    //proposal begin
    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 1 * 3600 * 24 * 1_000_000_000;
    
    //vote_user1 vote approve to proposal 0
    let orig_user_balance = proposal_user.account().unwrap().amount;
    let out_come = call!(
        vote_user1,
        referendum_contract.act_proposal(0, "approve".into(), Some("approve".to_string()))
    );    
    assert!(out_come.unwrap_json::<bool>());
    assert_eq!(proposal_user.account().unwrap().amount - orig_user_balance, to_yocto("10"));

    let proposal_info = view!(referendum_contract.get_proposal_info(0)).unwrap_json::<ProposalInfo>();

    assert_eq!(proposal_info.status, ProposalStatus::Approved);
    assert_eq!(proposal_info.lock_amount.0, 0);
    assert_eq!(proposal_info.vote_counts, [U128(to_yocto("30")), U128(0), U128(0), U128(to_yocto("60"))]);

    //vote_user1 vote reject to proposal 1
    let orig_user_balance = proposal_user.account().unwrap().amount;
    let out_come = call!(
        vote_user1,
        referendum_contract.act_proposal(1, "reject".into(), Some("reject".to_string()))
    );    
    assert!(out_come.unwrap_json::<bool>());
    assert_eq!(proposal_user.account().unwrap().amount - orig_user_balance, 0);

    let proposal_info = view!(referendum_contract.get_proposal_info(1)).unwrap_json::<ProposalInfo>();
    println!("{:?}", proposal_info);

    assert_eq!(proposal_info.status, ProposalStatus::InProgress);
    assert_eq!(proposal_info.lock_amount.0, to_yocto("10"));
    assert_eq!(proposal_info.vote_counts, [U128(0), U128(to_yocto("30")), U128(0), U128(to_yocto("60"))]);

    //vote_user2 vote reject to proposal 1
    let orig_user_balance = proposal_user.account().unwrap().amount;
    let out_come = call!(
        vote_user2,
        referendum_contract.act_proposal(1, "reject".into(), Some("reject".to_string()))
    );    
    assert!(out_come.unwrap_json::<bool>());
    assert_eq!(proposal_user.account().unwrap().amount - orig_user_balance, to_yocto("10"));

    let proposal_info = view!(referendum_contract.get_proposal_info(1)).unwrap_json::<ProposalInfo>();
    println!("{:?}", proposal_info);

    assert_eq!(proposal_info.status, ProposalStatus::Rejected);
    assert_eq!(proposal_info.lock_amount.0, 0);
    assert_eq!(proposal_info.vote_counts, [U128(0), U128(to_yocto("60")), U128(0), U128(to_yocto("60"))]);
}

#[test]
fn test_vote_no_proposal(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, vote_user1, _) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let vote_policy = VotePolicy::Relative(Rational{numerator:1, denominator:2}, Rational{numerator:1, denominator:2});

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), vote_policy.try_to_vec().unwrap().into(), 0, 24 * 60 * 60, 24 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    call!(
        vote_user1,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    let out_come = call!(
        vote_user1,
        referendum_contract.act_proposal(1, "approve".into(), Some("approve".to_string()))
    );    

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NO_PROPOSAL"));
}

#[test]
fn test_vote_not_votable(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, vote_user1, vote_user2) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let vote_policy = VotePolicy::Relative(Rational{numerator:1, denominator:2}, Rational{numerator:1, denominator:2});

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), vote_policy.try_to_vec().unwrap().into(), 0, 24 * 60 * 60, 24 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    call!(
        vote_user1,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    call!(
        vote_user2,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "5".to_string()),
        deposit = 1
    ).assert_success();

    let out_come = call!(
        vote_user1,
        referendum_contract.act_proposal(0, "approve".into(), Some("approve".to_string()))
    );    

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_PROPOSAL_NOT_VOTABLE"));
}

#[test]
fn test_vote_already_vote(){
    let (root, _, _, xref_contract, referendum_contract) = 
        init_env(true);

    let (proposal_user, vote_user1, vote_user2) = init_proposal_users(&root, &xref_contract, &referendum_contract);

    let vote_policy = VotePolicy::Relative(Rational{numerator:1, denominator:2}, Rational{numerator:1, denominator:2});

    call!(
        proposal_user,
        referendum_contract.add_proposal("test proposal".to_string(), "vote".into(), vote_policy.try_to_vec().unwrap().into(), 0, 24 * 60 * 60, 24 * 60 * 60),
        deposit = 10_000_000_000_000_000_000_000_000
    ).assert_success();

    call!(
        vote_user1,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "1".to_string()),
        deposit = 1
    ).assert_success();

    call!(
        vote_user2,
        xref_contract.ft_transfer_call(referendum_contract.valid_account_id(), to_yocto("30").into(), None, "5".to_string()),
        deposit = 1
    ).assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = view!(referendum_contract.contract_metadata()).unwrap_json::<ContractMetadata>().genesis_timestamp + 1 * 3600 * 24 * 1_000_000_000;

    let out_come = call!(
        vote_user1,
        referendum_contract.act_proposal(0, "approve".into(), Some("approve".to_string()))
    );   
    assert!(out_come.unwrap_json::<bool>());
    
    let out_come = call!(
        vote_user1,
        referendum_contract.act_proposal(0, "approve".into(), Some("approve".to_string()))
    );    

    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ALREADY_VOTED"));
}
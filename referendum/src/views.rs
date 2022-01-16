//! View functions for the contract.

use crate::*;
use crate::proposals::Vote;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ContractMetadata {
    /// the owner account id of contract
    pub owner_id: AccountId,
    /// accept lock token account id
    pub locked_token: AccountId,
    /// the launch timestamp in seconds
    pub genesis_timestamp_sec: u32,
    /// current session id (start from 0)
    pub cur_session_id: u32,
    /// current total ballot amount (calculate at call time)
    pub cur_total_ballot: U128,
    /// current locking token amount (include those expired but hasn't unlock by user)
    pub cur_lock_amount: U128,
    /// the availabe proposal id for new proposal
    pub last_proposal_id: u32,
    /// lock near amount for endorsement a proposal
    pub lock_amount_per_proposal: U128,
    /// current account number in contract
    pub account_number: u64,
    /// a list of [Relative, Absolute] in which each item is formated as 
    /// [{"numerator": n, "denominator": m}, {"numerator": n, "denominator": m}]
    pub vote_policy: Vec<VotePolicy>,
    /// in format as {"numerator": n, "denominator": m}
    pub nonsense_threshold: Rational,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ProposalInfo{
    pub id: u32,
    pub proposer: AccountId,
    /// near amount for endorsement
    pub lock_amount: U128,
    pub description: String,
    /// one of the following:
    /// "VotePolicy": {"Relative": [{"numerator": n, "denominator": m}, {"numerator": n, "denominator": m}]}
    /// "VotePolicy": {"Absolute": [{"numerator": n, "denominator": m}, {"numerator": n, "denominator": m}]}
    pub vote_policy: proposals::VotePolicy,
    /// currently would only be "Vote"
    pub kind: proposals::ProposalKind,
    /// one of the following:
    /// "WarmUp", "InProgress", "Approved", "Rejected", "Nonsense", "Expired"
    pub status: proposals::ProposalStatus,
    /// [Approve_count, Reject_count, Nonsense_count, Total_ballots]
    pub vote_counts: [U128; 4],
    /// The session this proposal is valid in
    pub session_id: u32,
    /// the start time = session_begin_time + start_offset
    pub start_offset_sec: u32,
    /// the proposal max valid period in seconds
    pub lasts_sec: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct AccountInfo {
    /// locked token (XREF) amount
    pub locking_amount: U128,
    /// ballot amount (calculate at call time)
    pub ballot_amount: U128,
    /// unlock at the begin of this session, meanwhile ballots reset to zero
    pub unlocking_session_id: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct HumanReadableAccountVote {
    pub proposal_id: u32,
    pub vote: Vote,
    pub amount: U128,
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone, Copy))]
pub struct SessionState {
    pub session_id: u32,
    pub expire_amount: U128,
}

impl From<SessionInfo> for SessionState {
    fn from(session_info: SessionInfo) -> Self {
        Self {
            session_id: session_info.session_id,
            expire_amount: session_info.expire_amount.into(),
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Return contract basic info
    pub fn contract_metadata(&self) -> ContractMetadata {
        let current_state = self.data();
        ContractMetadata {
            owner_id: current_state.owner_id.clone(),
            locked_token: current_state.locked_token.clone(),
            genesis_timestamp_sec: nano_to_sec(current_state.genesis_timestamp),
            cur_session_id: self.get_cur_session_id(),
            cur_total_ballot: self.calc_cur_ballots().into(),
            cur_lock_amount: current_state.cur_lock_amount.into(),
            last_proposal_id: current_state.last_proposal_id,
            lock_amount_per_proposal: U128(current_state.lock_amount_per_proposal),
            account_number: current_state.account_number,
            vote_policy: current_state.vote_policy.clone(),
            nonsense_threshold: current_state.nonsense_threshold.clone(),
        }
    }

    /// get single proposal current info
    pub fn get_proposal_info(&self, proposal_id: u32) -> Option<ProposalInfo>{
        if let Some(VersionedProposal::Default(proposal)) = self.data().proposals.get(&proposal_id).as_ref() {
            Some(ProposalInfo{
                id: proposal_id,
                proposer: proposal.proposer.clone(),
                lock_amount: U128(proposal.lock_amount),
                description: proposal.description.clone(),
                vote_policy: proposal.vote_policy.clone(),
                kind: proposal.kind.clone(),
                status: proposal.get_cur_status(self.data().genesis_timestamp),
                vote_counts: proposal.vote_counts.map(|v| U128(v)),
                session_id: proposal.session_id,
                start_offset_sec: nano_to_sec(proposal.start_offset),
                lasts_sec: nano_to_sec(proposal.lasts),
            })
        }else{
            None
        }
    }

    /// get proposals by session
    pub fn get_proposals_in_session(&self, session_id: u32) -> Vec<ProposalInfo> {
        let mut ret: Vec<ProposalInfo> = vec![];
        for id in self.get_proposal_ids_in_session(session_id) {
            if let Some(item) = self.get_proposal_info(id) {
                ret.push(item);
            }  
        }
        ret
    }

    pub fn get_proposal_ids_in_session(&self, session_id: u32) -> Vec<u32> {
        match self.data().proposal_ids_in_sessions.get(session_id as u64) {
            Some(proposal_ids) => proposal_ids,
            None => vec![]
        }
    }

    pub fn get_account_info(&self, account_id: ValidAccountId) -> Option<AccountInfo> {
        if let Some(vacc) = self.data().accounts.get(account_id.as_ref()) {
            match vacc {
                VAccount::Current(acc) => {
                    Some(AccountInfo {
                        locking_amount: acc.locking_amount.into(),
                        ballot_amount: acc.sync_ballot(self.get_cur_session_id()).into(),
                        unlocking_session_id: acc.unlocking_session_id,
                    })
                }
            }
        } else {
            None
        }
    }

    pub fn get_account_proposals_in_session(&self, account_id: ValidAccountId, session_id: u32) -> Vec<HumanReadableAccountVote> {
        let mut ret: Vec<HumanReadableAccountVote> = vec![];
        match self.data().proposal_ids_in_sessions.get(session_id as u64) {
            Some(proposal_ids) => {
                if let Some(vacc) = self.data().accounts.get(account_id.as_ref()) {
                    match vacc {
                        VAccount::Current(acc) => {
                            for proposal_id in proposal_ids {
                                if let Some(account_vote) = acc.proposals.get(&proposal_id).as_ref() {
                                    ret.push(HumanReadableAccountVote {
                                        proposal_id,
                                        vote: account_vote.vote.clone(),
                                        amount: account_vote.amount.into(),
                                    });
                                }
                            }
                        }
                    }
                }
            },
            None => {},
        }
        ret   
    }

    // TODO: maybe unnecessary to get session info
    pub fn get_session_state(&self, session_idx: usize) -> SessionState {
        self.data().sessions[session_idx].into()
    }
}
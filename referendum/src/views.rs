//! View functions for the contract.

use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractMetadata {
    pub owner_id: AccountId,
    pub locked_token: AccountId,
    pub genesis_timestamp_sec: u32,
    pub cur_session: usize,
    pub cur_total_ballot: U128,
    pub last_proposal_id: u64,
    pub lock_amount_per_proposal: U128,
    pub account_number: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInfo{
    pub proposer: AccountId,
    pub lock_amount: U128,
    pub description: String,
    pub vote_policy: proposals::VotePolicy,
    pub kind: proposals::ProposalKind,
    pub status: proposals::ProposalStatus,
    pub vote_counts: [U128; 4],
    pub session_id: u32,
    pub start_offset: Timestamp,
    pub lasts: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountInfo {
    pub locking_amount: U128,
    pub ballot_amount: U128,
    pub unlocking_session_id: u32,
}

impl From<VAccount> for AccountInfo {
    fn from(vacc: VAccount) -> Self {
        match vacc {
            VAccount::Current(acc) => {
                Self {
                    locking_amount: acc.locking_amount.into(),
                    ballot_amount: acc.ballot_amount.into(),
                    unlocking_session_id: acc.unlocking_session_id,
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
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
            cur_session: current_state.cur_session,
            cur_total_ballot: current_state.cur_total_ballot.into(),
            last_proposal_id: current_state.last_proposal_id,
            lock_amount_per_proposal: U128(current_state.lock_amount_per_proposal),
            account_number: current_state.account_number,
        }
    }

    pub fn get_proposal_info(&self, proposal_idx: u64) -> ProposalInfo{
        if let Some(VersionedProposal::Default(proposal)) = self.data().proposals.get(&proposal_idx){
            ProposalInfo{
                proposer: proposal.proposer,
                lock_amount: U128(proposal.lock_amount),
                description: proposal.description,
                vote_policy: proposal.vote_policy,
                kind: proposal.kind,
                status: proposal.status,
                vote_counts: proposal.vote_counts.map(|v| U128(v)),
                session_id: proposal.session_id,
                start_offset: proposal.start_offset,
                lasts: proposal.lasts,
            }
        }else{
            env::panic(b"Err_INVALID_PROPOSAL_IDX")
        }
    }

    pub fn get_proposal_ids_in_session(&self, session_id: u64) -> Vec<u64>{
        match self.data().proposal_ids_in_sessions.get(session_id) {
            Some(proposal_ids) => proposal_ids,
            None => vec![]
        }
    }

    pub fn get_user_base_info(&self, account_id: ValidAccountId) -> AccountInfo {
        self.data().accounts.get(account_id.as_ref()).unwrap().into()
    }

    pub fn get_session_state(&self, session_idx: usize) -> SessionState {
        self.data().sessions[session_idx].into()
    }
}
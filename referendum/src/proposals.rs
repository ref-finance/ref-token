use std::collections::HashMap;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::{log, AccountId, Balance, Gas, PromiseOrValue, Timestamp, Promise};
use near_sdk::serde::{Deserialize, Serialize};

use crate::*;
use crate::utils::Rational;


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PolicyType {
    Relative = 0x0,
    Absolute = 0x1,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VotePolicy {
    Relative(Rational, Rational),
    Absolute(Rational, Rational),
}

impl From<Vec<u8>> for VotePolicy {
    fn from(content: Vec<u8>) -> Self {
        VotePolicy::try_from_slice(&content).unwrap()
    }
}

impl VotePolicy {

    /// to see if the proposal goes to a final state
    pub fn judge(&self, aye: &Balance, nay: &Balance, remove: &Balance, total: &Balance, nonsense_threshold: &Rational) -> ProposalStatus {
        if nonsense_threshold.pass(remove, total) {
            ProposalStatus::Removed
        } else {
            match self {
                VotePolicy::Relative(limit, threshold) => {
                    let voted = aye + nay + remove;
                    if limit.pass(&voted, total) {
                        if threshold.pass(nay, &voted) {
                            ProposalStatus::Rejected
                        } else if threshold.pass(aye, &voted) {
                            ProposalStatus::Approved
                        } else {
                            ProposalStatus::InProgress
                        }
                    } else {
                        ProposalStatus::InProgress
                    }
                },
                VotePolicy::Absolute(pass_threshold, fail_threshold) => {
                    if fail_threshold.pass(nay, total) {
                        ProposalStatus::Rejected
                    } else if pass_threshold.pass(aye, total) {
                        ProposalStatus::Approved
                    } else {
                        ProposalStatus::InProgress
                    }
                },
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            VotePolicy::Relative(limit, threshold) => {
                limit.is_valid() && threshold.is_valid()
            },
            VotePolicy::Absolute(pass_threshold, fail_threshold) => {
                pass_threshold.is_valid() && fail_threshold.is_valid()
            },
        }
    }
    
}

/// Status of a proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    WarmUp,
    InProgress,
    /// If quorum voted yes, this proposal is successfully approved.
    Approved,
    /// If quorum voted no, this proposal is rejected. Bond is returned.
    Rejected,
    /// If quorum voted to remove (e.g. spam), this proposal is rejected and bond is not returned.
    /// Interfaces shouldn't show removed proposals.
    Removed,
    /// Expired after period of time.
    Expired,
}

/// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    /// Just a signaling vote, with no execution.
    Vote,
}

impl ProposalKind {
    /// Returns label of policy for given type of proposal.
    pub fn to_policy_label(&self) -> &str {
        match self {
            ProposalKind::Vote => "vote",
        }
    }
}

impl From<&str> for ProposalKind {
    fn from(kind: &str) -> Self {
        match kind {
            "vote" => {
                ProposalKind::Vote
            },
            _ => env::panic(b"ERR_NO_INVALID_PROPOSAL_KIND")
        }
    }
}

/// Set of possible action to take.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    /// Vote to approve given proposal or bounty.
    VoteApprove,
    /// Vote to reject given proposal or bounty.
    VoteReject,
    /// Vote to remove given proposal or bounty (because it's spam).
    VoteRemove,
}

impl From<&str> for Action {
    fn from(action: &str) -> Self {
        match action {
            "approve" => {
                Action::VoteApprove
            },
            "reject" => {
                Action::VoteReject
            },
            "remove" => {
                Action::VoteRemove
            },
            _ => env::panic(b"ERR_NO_INVALID_ACTION_KIND")
        }
    }
}

impl Action {
    pub fn to_policy_label(&self) -> String {
        format!("{:?}", self)
    }
}

/// Votes recorded in the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Vote {
    Approve = 0x0,
    Reject = 0x1,
    Remove = 0x2,
}

impl From<Action> for Vote {
    fn from(action: Action) -> Self {
        match action {
            Action::VoteApprove => Vote::Approve,
            Action::VoteReject => Vote::Reject,
            Action::VoteRemove => Vote::Remove,
        }
    }
}

/// Proposal that are sent to this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    /// Original proposer.
    pub proposer: AccountId,
    /// The locked near as the endorsement of this proposal
    pub lock_amount: Balance,
    /// Description of this proposal.
    pub description: String,
    /// Voting rule details
    pub vote_policy: VotePolicy,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: Aye / Nay / Remove / Total.
    pub vote_counts: [Balance; 4],
    /// Session ID for voting period.
    pub session_id: u32,
    /// the nano seconds of voting begin time after the session begin for the proposal, 
    /// before this time, proposer can remove this immediately.
    pub start_offset: Timestamp,
    /// the nano seconds of voting lasts after start_offset for the proposal, 
    /// An inprogress poposal would change to expired after it.
    /// The (start_offset+lasts) should less than SESSION_INTERVAL.
    pub lasts: Timestamp,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedProposal {
    Default(Proposal),
}

impl From<VersionedProposal> for Proposal {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::Default(p) => p,
        }
    }
}

impl Proposal {
    /// Adds votes to proposal.
    /// pre-requisite: status == InProgress
    pub fn update_votes(
        &mut self,
        vote: &Vote,
        amount: &Balance,
        total: &Balance,
        nonsense_threshold: &Rational,
    ) {
        self.vote_counts[vote.clone() as usize] += amount;
        self.vote_counts[3] = total.clone();

        self.status = self.vote_policy.judge(
            &self.vote_counts[0], 
            &self.vote_counts[1], 
            &self.vote_counts[2], 
            &self.vote_counts[3],
            nonsense_threshold,
        );
    }

    pub fn get_cur_status(&self, genesis_ts: Timestamp) -> ProposalStatus {
        let cur_ts = env::block_timestamp();
        let session_start = genesis_ts + self.session_id as u64 * SESSION_INTERMAL;
        let begin_ts = session_start + self.start_offset;
        let end_ts = begin_ts + self.lasts;
        match self.status {
            ProposalStatus::WarmUp => {
                if cur_ts > end_ts {
                    ProposalStatus::Expired
                } else if cur_ts > begin_ts {
                    ProposalStatus::InProgress
                } else {
                    self.status.clone()
                }
            },
            ProposalStatus::InProgress => {
                if cur_ts > end_ts {
                    ProposalStatus::Expired
                } else {
                    self.status.clone()
                }
            },
            _ => self.status.clone(),
        }
    }
}

impl Contract {
    
}

#[near_bindgen]
impl Contract {
    /// Add proposal to this DAO.
    #[payable]
    pub fn add_proposal(&mut self, description: String, kind: ProposalKind, policy_type: PolicyType, session_id: u32, start_offset_sec: u32, lasts_sec: u32) -> u64 {
        // check point
        self.fresh_sessions();

        let proposer = env::predecessor_account_id();

        // check and lock deposit
        let deposit_amount = env::attached_deposit();
        assert!(deposit_amount >= self.data().lock_amount_per_proposal, "ERR_NOT_ENOUGH_LOCK_NEAR");
        if deposit_amount > self.data().lock_amount_per_proposal {
            Promise::new(proposer.clone()).transfer(deposit_amount - self.data().lock_amount_per_proposal);
        }

        // TODO: check time validation, session_id gte cur_session_id, (session_id.begin_ts+start_offset+lasts) lt (session_id+1).begin_ts
        let current_session_id = self.data().sessions[self.data().cur_session].session_id;
        assert!(session_id >= current_session_id, "ERR_SESSION_ID_NEED_GE_CURRENT_SESSION_ID");
        let base_timestamp = self.data().genesis_timestamp + SESSION_INTERMAL * session_id as u64;
        assert!((base_timestamp + sec_to_nano(start_offset_sec)) > env::block_timestamp(), "ERR_PROPOSAL_START_TIME_NEED_GE_CURRENT_TIME");
        assert!((base_timestamp + sec_to_nano(start_offset_sec + lasts_sec)) < base_timestamp + SESSION_INTERMAL, "ERR_PROPOSAL_END_TIME_NEED_LE_NEXT_SESSION_BEGIN_TIME");

        let ps = Proposal {
            proposer,
            lock_amount: self.data().lock_amount_per_proposal,
            description,
            vote_policy: self.data().vote_policy.get(policy_type as usize).unwrap().clone(),
            kind,
            status: ProposalStatus::WarmUp,
            vote_counts: [0; 4],
            session_id,
            start_offset: sec_to_nano(start_offset_sec),
            lasts: sec_to_nano(lasts_sec),
        };

        // actually add proposal to this DAO
        let id = self.data().last_proposal_id;
        self.data_mut().proposals.insert(&id, &VersionedProposal::Default(ps));
        self.data_mut().last_proposal_id += 1;

        id
    }

    /// proposer can call this to remove proposal before start time.
    pub fn remove_proposal(&mut self, id: u64) -> bool {
        // sync point
        self.fresh_sessions();
        let proposal: Proposal = self.data().proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        assert_eq!(proposal.proposer, env::predecessor_account_id(), "ERR_NOT_ALLOW");
        let cur_status = proposal.get_cur_status(self.data().genesis_timestamp);
        match cur_status {
            ProposalStatus::WarmUp => {
                if proposal.lock_amount > 0 {
                    Promise::new(proposal.proposer.clone()).transfer(proposal.lock_amount);
                }
                self.data_mut().proposals.remove(&id);
                true
            },
            _ => false,
        }
    }

    /// When a proposal expired, the proposer can call this to redeem locked near
    pub fn redeem_near_in_expired_proposal(&mut self, id: u64) -> bool {
        // sync point
        self.fresh_sessions();
        let mut proposal: Proposal = self.data().proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        assert_eq!(proposal.proposer, env::predecessor_account_id(), "ERR_NOT_ALLOW");
        let cur_status = proposal.get_cur_status(self.data().genesis_timestamp);
        proposal.status = cur_status;
        if proposal.lock_amount > 0 && proposal.status == ProposalStatus::Expired {
            Promise::new(proposal.proposer.clone()).transfer(proposal.lock_amount);
            proposal.lock_amount = 0;
            self.data_mut().proposals.insert(&id, &VersionedProposal::Default(proposal));
            true
        } else {
            false
        }
    }

    /// Act on given proposal by id, if permissions allow.
    /// Memo is logged but not stored in the state. Can be used to leave notes or explain the action.
    pub fn act_proposal(&mut self, id: u64, action: Action, memo: Option<String>) -> bool {
        // sync point
        self.fresh_sessions();

        let vote: Vote = action.into();

        let mut proposal: Proposal = self.data().proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let cur_status = proposal.get_cur_status(self.data().genesis_timestamp);
        proposal.status = cur_status;
        
        // check proposal is inprogress
        assert_eq!(proposal.status, ProposalStatus::InProgress, "ERR_PROPOSAL_NOT_VOTABLE");
        
        let account_id = env::predecessor_account_id();

        // get vote amount for this action
        let ballot_amount = self.internal_vote(&account_id, id, &vote);

        // update and judge proposal result
        proposal.update_votes(&vote, &ballot_amount, &self.data().cur_total_ballot, &self.data().nonsense_threshold);

        if proposal.status == ProposalStatus::Approved || proposal.status == ProposalStatus::Rejected {
            // return lock near to proposer
            Promise::new(proposal.proposer.clone()).transfer(proposal.lock_amount);
            proposal.lock_amount = 0;
        }

        self.data_mut().proposals.insert(&id, &VersionedProposal::Default(proposal));

        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }

        true
    }
}
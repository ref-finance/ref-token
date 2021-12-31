/*!
* REF referendum contract
*
*/
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Timestamp, BorshStorageKey};
use proposals::VotePolicy;

use crate::session::SessionInfo;
use crate::account::VAccount;
use crate::proposals::VersionedProposal;
use crate::utils::*;

mod session;
mod proposals;
mod account;
mod utils;
mod owner;
mod storage_impl;
mod views;

near_sdk::setup_alloc!();

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Accounts,
    Proposals,
    ProposalIdsInSession,
    AccountProposals {account_id: AccountId},
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractData {

    // owner of this contract
    owner_id: AccountId,

    // which token used for locking
    locked_token: AccountId,

    // the genesis timestamp
    genesis_timestamp: Timestamp,

    // maintains a global session circle array
    sessions: [SessionInfo; MAX_SESSIONS],

    // each session contains proposal id array
    proposal_ids_in_sessions: Vector<Vec<u32>>,

    // current session idx in sessions array
    cur_session: usize,

    // total ballot amount in current session
    cur_total_ballot: Balance,
    // total lock token amount
    cur_lock_amount: Balance,

    accounts: LookupMap<AccountId, VAccount>,
    account_number: u64,

    // the global vote policy
    vote_policy: Vec<VotePolicy>,

    /// Last available id for the proposals.
    pub last_proposal_id: u32,
    /// Proposal map from ID to proposal information.
    pub proposals: LookupMap<u32, VersionedProposal>,
    
    /// limits
    pub lock_amount_per_proposal: Balance,
    pub nonsense_threshold: Rational,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VContractData {
    Current(ContractData),
}

impl VContractData {}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    data: VContractData,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId, token_id: ValidAccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            data: VContractData::Current(ContractData {
                owner_id: owner_id.into(),
                locked_token: token_id.into(),
                genesis_timestamp: env::block_timestamp() + DEFAULT_GENESIS_OFFSET,
                sessions: [SessionInfo::default(); MAX_SESSIONS],
                proposal_ids_in_sessions: Vector::new(StorageKeys::ProposalIdsInSession),
                cur_session: 0,
                cur_total_ballot: 0,
                cur_lock_amount: 0,
                accounts: LookupMap::new(StorageKeys::Accounts),
                account_number: 0,
                vote_policy: vec![DEFAULT_VP_RELATIVE, DEFAULT_VP_ABSOLUTE],
                last_proposal_id: 0,
                proposals: LookupMap::new(StorageKeys::Proposals),
                lock_amount_per_proposal: DEFAULT_LOCK_NEAR_AMOUNT_FOR_PROPOSAL,
                nonsense_threshold: DEFAULT_NONSENSE_THRESHOLD,
            }),
        }
    }
}

impl Contract {
    fn data(&self) -> &ContractData {
        match &self.data {
            VContractData::Current(data) => data,
        }
    }

    fn data_mut(&mut self) -> &mut ContractData {
        match &mut self.data {
            VContractData::Current(data) => data,
        }
    }

    fn has_launch(&self) -> bool {
        return env::block_timestamp() > self.data().genesis_timestamp
    }

    fn get_cur_session_id(&self) -> u32 {
        let cur_session_id = (env::block_timestamp() - self.data().genesis_timestamp) / SESSION_INTERMAL;
        cur_session_id as u32
    }
}

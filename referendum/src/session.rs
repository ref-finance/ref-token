//! Session stores information per session

use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::Balance;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Default)]
pub struct SessionInfo {
    pub session_id: u32,
    pub expire_amount: Balance,
}

impl Contract {
    /// get newest cur ballots
    pub(crate) fn calc_cur_ballots(&self) -> Balance {
        if self.has_launch() {
            let cur_session_id = self.get_cur_session_id();
            if self.data().sessions[1].session_id == 0 {
                // hasn't initialized
                0
            } else {
                // get real ballot
                let head = self.data().cur_session;
                let mut ballot = self.data().cur_total_ballot;
                for i in 0..MAX_SESSIONS {
                    let idx = (i + head) % MAX_SESSIONS;
                    let session = self.data().sessions[idx].clone();
                    if session.session_id < cur_session_id {
                        // expire ballot
                        ballot -= session.expire_amount;
                    } else {
                        break;
                    }
                }
                ballot
            }
        } else {
            // before launch, ballot amount is 0
            0
        }
    }

    /// update sessions.
    pub(crate) fn fresh_sessions(&mut self) {
        self.assert_launch();
        let cur_session_id = self.get_cur_session_id();

        let head = self.data().cur_session;
        if self.data().sessions[1].session_id == 0 {
            // initialize session info
            for i in 0..MAX_SESSIONS {
                self.data_mut().sessions[i].session_id = cur_session_id + i as u32;
                self.data_mut().proposal_ids_in_sessions.push(&vec![]);
            }
        } else {
            // checkpoint logic
            for i in 0..MAX_SESSIONS {
                let idx = (i + head) % MAX_SESSIONS;
                let session = self.data().sessions[idx].clone();
                if session.session_id < cur_session_id {
                    // expire ballot
                    self.data_mut().cur_total_ballot -= session.expire_amount;
                    // prepare for new session
                    self.data_mut().sessions[idx].expire_amount = 0;
                    self.data_mut().sessions[idx].session_id =
                        session.session_id + MAX_SESSIONS as u32;
                    self.data_mut().proposal_ids_in_sessions.push(&vec![]);
                } else {
                    // spin to the new head
                    self.data_mut().cur_session = idx;
                    break;
                }
            }
        }
    }

    pub(crate) fn add_proposal_to_session(&mut self, proposal_id: u32, session_id: u32) {
        let mut proposal_ids = self
            .data()
            .proposal_ids_in_sessions
            .get(session_id as u64)
            .unwrap();
        proposal_ids.push(proposal_id);
        self.data_mut()
            .proposal_ids_in_sessions
            .replace(session_id as u64, &proposal_ids);
    }

    pub(crate) fn remove_proposal_from_session(&mut self, proposal_id: u32, session_id: u32) {
        let mut proposal_ids = self
            .data()
            .proposal_ids_in_sessions
            .get(session_id as u64)
            .unwrap();
        proposal_ids.retain(|&x| x != proposal_id);
        self.data_mut()
            .proposal_ids_in_sessions
            .replace(session_id as u64, &proposal_ids);
    }
}

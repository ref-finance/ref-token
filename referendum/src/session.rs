//! Session stores information per session 

use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::Balance;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Default)]
pub struct SessionInfo {
    pub session_id: u32,
    // pub expire_time: Timestamp,
    pub expire_amount: Balance,
}

impl Contract {
    /// update sessions.
    pub(crate) fn fresh_sessions(&mut self) {
        self.assert_launch();
        let cur_session_id = (env::block_timestamp() - self.data().genesis_timestamp) / SESSION_INTERMAL;
        
        let head = self.data().cur_session;
        if self.data().sessions[1].session_id == 0 {
            // TODO: need initiate all sessions according to genesis timestamp
            for i in 0..MAX_SESSIONS {
                self.data_mut().sessions[i].session_id = cur_session_id as u32 + i as u32;
            }
        } else {
            // checkpoint logic
            for i in 0..MAX_SESSIONS {
                let idx = (i + head) % MAX_SESSIONS; 
                let session = self.data().sessions[idx].clone();
                if (session.session_id as u64) < cur_session_id {
                    // expire ballot
                    self.data_mut().cur_total_ballot -= session.expire_amount;
                    // prepare for new session
                    self.data_mut().sessions[idx].expire_amount = 0;
                    self.data_mut().sessions[idx].session_id = session.session_id + MAX_SESSIONS as u32;
                } else {
                    // spin to the new head
                    self.data_mut().cur_session = idx;
                    break;
                }
            }
        }
    }
}
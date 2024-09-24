/*
    Reference:
    In search of an understandable consensus algorithm
    https://raft.github.io/raft.pdf
*/

use state::NodeState;

use crate::raft::*;

#[derive(Debug)]
pub struct Raft {
    persist_state: NodeState,
}

impl Raft {
    pub fn init() -> Raft {
        return Raft {
            persist_state: NodeState::init_state(),
        };
    }

    pub fn heart_beats() {
        // Abstraction over connection to send
        // heartbeats to its connections
        todo!()
    }

    pub fn request_vote() {
        todo!()
    }
}

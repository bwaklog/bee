// A serer will show 3 primary states of
// - follower, candidate and Leader.

use rmp;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::raft::state::state_helpers::gen_rand_id;
use crate::store::{self, LogEntry};
use crate::utils::helpers::{self};

pub type NodeTerm = u64;
pub type NodeId = u64;
pub type LogIndex = usize;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum State {
    FOLLOWER,
    CANDIDATE,
    LEADER,
}

pub type RaftStateError<T> = Result<T, StateErrors>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum StateErrors {
    LoadPersistedStateError(io::Error),
    SerializationError(serde_json::Error),
    MsgPackWriteError(rmp::encode::ValueWriteError),
    HelperErrorResult(helpers::HelperErrors),
}

impl From<io::Error> for StateErrors {
    fn from(value: io::Error) -> Self {
        StateErrors::LoadPersistedStateError(value)
    }
}

impl From<serde_json::Error> for StateErrors {
    fn from(value: serde_json::Error) -> Self {
        StateErrors::SerializationError(value)
    }
}

impl From<rmp::encode::ValueWriteError> for StateErrors {
    fn from(value: rmp::encode::ValueWriteError) -> Self {
        StateErrors::MsgPackWriteError(value)
    }
}

impl From<helpers::HelperErrors> for StateErrors {
    fn from(value: helpers::HelperErrors) -> Self {
        StateErrors::HelperErrorResult(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeState {
    pub node_id: u64,

    // have a mutex lock over the NodeState
    // while modifying these values
    pub current_term: NodeTerm,
    pub voted_for: Option<NodeId>,
    pub log: Vec<store::LogEntry>,

    pub node_state: State,
    //
    // Volatile server states
    current_leader: Option<NodeId>,

    // Volatile states on _all servers_
    // monotonically increases
    commit_index: LogIndex,
    last_applied: LogIndex,

    // Volatile leader states
    votes_recieved: Vec<NodeId>,

    // volatile states on leader
    // NOTE: idts vec is a good
    sent_length: Vec<store::LogEntry>,
    ack_length: Vec<store::LogEntry>,
}

impl NodeState {
    // While a node starts up after a crash or from shutdown state, we
    // need to read load the persisted states on disk

    // Better ase results here, Options are ugly and
    // not descriptive enough
    // recovers -> currentTerm, NodeId votedFor and the Log of entries
    // pub fn recover_state() -> Option<(NodeTerm, NodeId, Vec<store::LogEntry>)> {
    #[allow(unused_variables)]
    pub fn recover_state(
        state_path: PathBuf,
    ) -> RaftStateError<(NodeTerm, NodeId, Vec<store::LogEntry>)> {
        // assuming we parse a structured file
        // at this point

        let state_file = File::open(state_path.as_path())?;

        todo!()
    }

    #[allow(unused)]
    pub fn assert_state(&mut self) {
        assert_eq!(self.node_state, State::FOLLOWER);
        assert_eq!(self.commit_index, 0);
        assert_eq!(self.last_applied, 0);
        assert_eq!(self.current_leader, None);
        assert_eq!(self.sent_length.len(), 0);
        assert_eq!(self.ack_length.len(), 0);
    }

    #[allow(unused)]
    pub fn persist_state(&mut self, state_path: PathBuf) -> RaftStateError<()> {
        let persisted_state_json: (NodeTerm, Option<NodeId>, Vec<store::LogEntry>) =
            (self.current_term, self.voted_for, self.log.clone());
        let state_serialised = serde_json::to_vec(&persisted_state_json)?;
        let mut state_file = File::open(state_path.as_path())?;
        state_file.write_all(&state_serialised)?;
        Ok(())
    }

    #[allow(unused)]
    pub fn start_election(&mut self) {
        if self.node_state != State::FOLLOWER {
            return;
        }
        self.node_state = State::CANDIDATE;
        self.current_term += 1;
    }

    // initialization of a node
    // NOTE: passing down state_path down two functions
    pub fn init_state(state_path: PathBuf) -> Arc<Mutex<NodeState>> {
        let mut node_term: NodeTerm = 0;
        let mut voted_for: Option<NodeId> = None;
        let log: Vec<LogEntry>;

        // type LogIndex = u32;
        let recover = NodeState::recover_state(state_path);

        match recover {
            Ok((term, node_vote, recovered_log)) => {
                node_term = term;
                voted_for = Some(node_vote);

                // FIX
                log = recovered_log;
            }
            _ => {
                log = Vec::new();
            }
        }

        debug!(
            log = tracing::field::debug(&log),
            node_term = node_term,
            voted_for = voted_for,
            "Recovered config",
        );

        // NOTE: my god too many vecs (allocations!)
        return Arc::new(Mutex::new(NodeState {
            node_id: gen_rand_id(),
            current_term: node_term,
            voted_for,
            node_state: State::FOLLOWER,
            current_leader: None,
            log,
            commit_index: 0,
            last_applied: 0,
            votes_recieved: Vec::new(),
            sent_length: Vec::new(),
            ack_length: Vec::new(),
        }));
    }

    // async functions for RPC services

    #[allow(unused)]
    pub async fn echo(&self, input: String) -> String {
        format!("{input}")
    }

    #[allow(unused)]
    // return type -> NodeTerm, isLeader
    pub fn get_state(&self) -> Option<(NodeTerm, bool, NodeId)> {
        let node_term = self.current_term;
        let is_leader: bool = self.node_state == State::LEADER;
        let node_id = self.node_id;
        return Some((node_term, is_leader, node_id));
    }

    #[allow(unused)]
    // NOTE:
    // This is a very bad idea, I don't know how else to
    // have a full context of the node without this
    //
    // Can't hold a lock for long enough while making
    // an async call
    pub fn node_snapshot(&self) -> Option<NodeState> {
        return Some(self.clone());
    }

    // NOTE:
    // called when node transitions to a candidate
    // when the election timer times out
    pub async fn transition_to_candidate(&mut self) -> bool {
        if self.node_state == State::CANDIDATE {
            self.current_term += 1;
            return true;
        }
        if self.voted_for == None {
            self.current_term += 1;
            if self.node_state != State::FOLLOWER {
                return false;
            }
            self.voted_for = Some(self.node_id);
            self.node_state = State::CANDIDATE;
            return true;
        }
        return false;
    }

    pub fn transition_to_leader(&mut self) -> bool {
        self.node_state = State::LEADER;
        return true;
    }
}

pub mod state_helpers {
    use rand::Rng;
    pub fn gen_rand_id() -> u64 {
        let mut rng = rand::thread_rng();
        let val = rng.gen_range(1..1000);
        val
    }
}

// #[cfg(test)]
// mod tests {
//     use std::path::PathBuf;

//     use crate::storage::{kv::Operation, store::*};
//     use crate::utils::helpers::{parse_config, HelperErrorResult};

//     use super::{NodeState, RaftStateError};

//     fn create_test_state_with_path(conf_path: PathBuf) -> HelperErrorResult<NodeState> {
//         let config = parse_config(conf_path)?;
//         let raft_sm = NodeState::init_state(config.store.local_path);
//         Ok(raft_sm)
//     }

//     #[test]
//     fn test_init() -> RaftStateError<()> {
//         let raft_sm = create_test_state_with_path(PathBuf::from("./tests/config.yml"))?;

//         assert!(raft_sm.commit_index == 0);
//         assert!(raft_sm.last_applied == 0);
//         assert!(raft_sm.voted_for == None);
//         Ok(())
//     }

//     #[test]
//     fn log_entry_append() -> HelperErrorResult<()> {
//         let mut raft_sm = create_test_state_with_path(PathBuf::from("./tests/config.yml"))?;
//         // dbg!(&raft_sm);
//         let log_entry_dummy =
//             LogEntry::new_entry(Operation::SET, 5, Data::STRING(String::from("hello")), 1);
//         // dbg!(&log_entry_dummy);
//         println!("Dummy log entry: {:?}", log_entry_dummy);

//         raft_sm.log.push(log_entry_dummy.clone());

//         debug_assert_eq!(raft_sm.log.len(), 1);
//         debug_assert_eq!(raft_sm.log.get(0).unwrap(), &log_entry_dummy);

//         Ok(())
//     }
// }

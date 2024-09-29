// A serer will show 3 primary states of
// - follower, candidate and Leader.

use std::{error, sync};

use crate::store::{self, LogEntry};

pub type NodeTerm = u32;
pub type NodeId = u32;
pub type LogIndex = u32;

#[derive(Debug, PartialEq, Eq)]
enum State {
    FOLLOWER,
    CANDIDATE,
    LEADER,
}

#[derive(Debug)]
pub struct NodeState {
    // have a mutex lock over the NodeState
    // while modifying these values
    current_term: NodeTerm,
    voted_for: Option<NodeId>,
    log: Vec<store::LogEntry>,

    node_state: State,
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

    // Better use results here, Options are ugly and
    // not descriptive enough
    // recovers -> currentTerm, NodeId votedFor and the Log of entries
    pub fn recover_state() -> Option<(NodeTerm, NodeId, Vec<LogEntry>)> {
        // assuming we parse a structured file
        // at this point
        None
    }

    pub fn persist_state() -> Result<(), std::io::Error> {
        todo!()
    }

    pub fn start_election(&mut self) {
        if self.node_state != State::FOLLOWER {
            return;
        }
        self.node_state = State::CANDIDATE;
        self.current_term += 1;

        // start election timer
    }

    // initialization of a node
    pub fn init_state() -> NodeState {
        let mut node_term: NodeTerm = 0;
        let mut voted_for: Option<NodeId> = None;
        let log: Vec<LogEntry>;

        // type LogIndex = u32;
        let recover: Option<(NodeTerm, NodeId, Vec<LogEntry>)> = NodeState::recover_state();

        match recover {
            Some((term, node_vote, recovered_log)) => {
                node_term = term;
                voted_for = Some(node_vote);

                // FIX
                log = recovered_log;
            }
            _ => {
                log = Vec::new();
            }
        }

        // NOTE: my god too many vecs (allocations!)
        return NodeState {
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
        };
    }

    pub fn get_state(&mut self, mu: sync::Mutex<&mut NodeState>) -> Option<(NodeTerm, bool)> {
        let lock = mu.lock();
        match lock {
            Ok(guard) => {
                let node_term = guard.current_term;
                let is_leader: bool = guard.node_state == State::LEADER;
                return Some((node_term, is_leader));
            }
            Err(err) => {
                println!("Failed to get lock: {err}");
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::{kv::Operation, store::*};

    use super::NodeState;

    #[test]
    #[ignore = "not implemented"]
    fn test_init() {
        let raft_sm = NodeState::init_state();
        assert!(raft_sm.commit_index == 0);
        assert!(raft_sm.last_applied == 0);
        assert!(raft_sm.voted_for == None);

        todo!("finish tests for nextIndex and matchIndex");
    }

    #[test]
    fn log_entry_append() {
        let mut raft_sm = NodeState::init_state();

        let log_entry_dummy =
            LogEntry::new_entry(Operation::SET, 5, Data::STRING(String::from("hello")), 1);

        raft_sm.log.push(log_entry_dummy.clone());

        debug_assert_eq!(raft_sm.log.len(), 1);
        debug_assert_eq!(raft_sm.log.get(0).unwrap(), &log_entry_dummy);
    }
}

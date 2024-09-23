/*
    Reference:
    In search of an understandable consensus algorithm
    https://raft.github.io/raft.pdf
*/

use crate::raft::*;
use crate::storage::store;

pub type NodeId = u32;
pub type LogIndex = u32;

// NOTE: is it better to have this state as
// an enum? does this decouple the transition
#[derive(Debug)]
enum State {
    FOLLOWER,
    CANDIDATE,
    LEADER,
}

// Describing the persisting state on
// all nodes
#[derive(Debug)]
struct PersistedState {
    node_id: NodeId,
    state: State,
    current_term: u32,

    log: Vec<store::LogEntry>,
}

impl PersistedState {
    fn init_state() -> PersistedState {
        // read from a dump file to recover from a point
        return PersistedState {
            node_id: util::nodeid_rand_u64(),
            state: State::FOLLOWER,
            current_term: 0,
            log: Vec::new(),
        };
    }
}

#[derive(Debug)]
pub struct Raft {
    persist_state: PersistedState,

    // State
    current_term: u32,
    voted_for: Option<NodeId>,

    // Volatile states on _all servers_
    // monotonically increases
    commit_index: LogIndex,
    last_applied: LogIndex,

    // volatile states on leader
    // NOTE: idts vec is a good
    next_index: Vec<store::LogEntry>,
    match_index: Vec<store::LogEntry>,
}

impl Raft {
    pub fn init() -> Raft {
        let mut persist_state = PersistedState::init_state();

        return Raft {
            persist_state,
            current_term: 0,
            voted_for: None,

            commit_index: 0,
            last_applied: 0,
            // Volatile states on server
            // next_index -> sentLen< ... >
            // match_index -> acknowledgedLen < ... >
            next_index: Vec::new(),
            match_index: Vec::new(),
        };
    }

    pub fn heart_beats() {
        // Abstraction over connection to send
        // heartbeats to its connections
    }

    pub fn request_vote() {}
}

#[cfg(test)]
mod tests {
    use crate::storage::{kv::Operation, store::*};

    use super::Raft;

    #[test]
    fn test_init() {
        panic!()
    }

    #[test]
    fn log_entry_append() {
        let mut raft_sm = Raft::init();

        let log_entry_dummy =
            LogEntry::new_entry(Operation::SET, 5, Data::STRING(String::from("hello")), 1);

        raft_sm.persist_state.log.push(log_entry_dummy.clone());

        debug_assert_eq!(raft_sm.persist_state.log.len(), 1);
        debug_assert_eq!(raft_sm.persist_state.log.get(0).unwrap(), &log_entry_dummy);
        // dbg!(raft_sm.persist_state);
    }
}

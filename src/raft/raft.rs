/*
    Reference:
    In search of an understandable consensus algorithm
    https://raft.github.io/raft.pdf
*/

use conn::ConnectionLayer;

use state::NodeState;

use crate::raft::*;
use crate::utils::helpers;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Raft {
    state: NodeState,
    config: helpers::RaftConfig,
    pub conn: ConnectionLayer,
}

#[allow(unused_mut, dead_code)]
impl Raft {
    pub fn init(raft_config: helpers::RaftConfig) -> Raft {
        // idealy this should be based on a cli flag to read
        // NOTE: persist_path does not implement Copy
        let mut state = NodeState::init_state(raft_config.persist_path.clone());
        state.assert_state();

        let mut conn = ConnectionLayer::init_layer(&raft_config.listener_addr);

        return Raft {
            state,
            conn,
            config: raft_config,
        };
    }

    pub fn heart_beats() {
        // Abstraction over connection to send
        // heartbeats to its connections
        todo!()
    }

    pub fn listen_leader() {
        // run asynchronously in background on intialization
        // when suspected leader failure, increase the term and
        // transition to a candidate node
        todo!()
    }

    pub fn request_vote() {
        todo!()
    }
}

#[allow(dead_code)]
pub mod raft_helpers {
    use rand::Rng;

    pub fn gen_rand_timeout() -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(200..=500)
    }
}

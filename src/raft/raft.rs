//
// Reference:
// In search of an understandable consensus algorithm
// https://raft.github.io/raft.pdf
//

use core::time;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Arc;

use crate::raft::state::{NodeState, NodeTerm};
use raft::conn::ConnectionLayer;
use raft::state::NodeId;
use raft_helpers::gen_rand_timeout;
use tokio::sync::{self, Mutex};
use tracing::{debug, info, warn};

use crate::raft;
use crate::utils::helpers;

use super::rpc;
use super::state::State;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Raft {
    state: Arc<Mutex<NodeState>>,
    config: helpers::RaftConfig,
    timeout: u64,
    // pub conn: ConnectionLayer,
}

#[allow(unused, dead_code)]
impl Raft {
    pub async fn node_daemon(&mut self) {
        // this is an asynchronous program running in the background
        // that will handle the states of the node
        // - timeouts (election & follower)
        // - waiting as a listener on RPC calls from a leader
        //   or a follower

        let (mut timeout_tx, mut timeout_rx): (
            sync::mpsc::Sender<String>,
            sync::mpsc::Receiver<String>,
        ) = tokio::sync::mpsc::channel(128);
        let (mut connlayer_tx, mut connlayer_rx): (
            sync::mpsc::Sender<ConnectionLayer>,
            sync::mpsc::Receiver<ConnectionLayer>,
        ) = tokio::sync::mpsc::channel(128);

        let timeout = Arc::new(self.timeout);
        let timeout_tx_arc = Arc::new(timeout_tx);
        let addr = Arc::new(Mutex::new(self.config.clone().listener_addr));
        let conn_sock_addrs = Arc::new(self.config.clone().connections);

        // FIX
        let node_state = Arc::clone(&self.state);

        let addr_mutex = addr.clone();

        let state_clone = Arc::clone(&node_state);

        // NOTE: RPC Listener
        tokio::spawn(async move {
            let node_addr = addr_mutex.lock().await;
            let state = Arc::clone(&state_clone);
            let mut conn = ConnectionLayer::init_layer(&node_addr, state)
                .await
                .expect("couldnt initalize a Raft Service");
            connlayer_tx.send(conn).await.unwrap();
        });

        let connections = conn_sock_addrs.clone();
        let state_clone = Arc::clone(&node_state);

        tokio::spawn(async move {
            let tx = Arc::clone(&timeout_tx_arc);
            let rpc_timeout = timeout.clone();
            let state = Arc::clone(&state_clone);
            let safe_state = state.lock().await;
            let node_id: u64 = safe_state.node_id;

            drop(safe_state);

            loop {
                tokio::time::sleep(time::Duration::from_secs(rpc_timeout.as_ref().to_owned()))
                    .await;

                tx.send("heartbeat timeout".to_owned()).await.unwrap();

                info!("Leader heartbeat timed out");

                let mut node_state_safe = state.lock().await;
                let mut node_state = node_state_safe.deref_mut();

                if &node_state.node_state == &State::FOLLOWER {
                    if node_state.transition_to_candidate().await == false {
                        warn!("Tried transitioning to candidate failed");
                    } else {
                        info!("Transition to candidate");
                        info!("Initiating leader election for Node {}", node_state.node_id);
                        let res =
                            Raft::start_leader_election(Arc::clone(&connections), node_state).await;
                    }
                } else if &node_state.node_state == &State::CANDIDATE {
                    // start leader election in a higher term
                    info!(
                        "Already a candidate. Initiating leader election for Node {} in a new term",
                        node_state.node_id
                    );
                    if node_state.transition_to_candidate().await == false {
                        warn!("Failed to increase term and transition")
                    } else {
                        let res =
                            Raft::start_leader_election(Arc::clone(&connections), node_state).await;
                    }
                } else if &node_state.node_state == &State::LEADER {
                    // Logic
                }
            }
        });

        loop {
            tokio::select! {
                Some(timeout_msg) = timeout_rx.recv() => {
                    // debug!("heartbeat wait timeout: {timeout_msg}");
                }
                Some(conn) = connlayer_rx.recv() => {
                    // debug!("{:?}", conn);
                }
            }
        }
    }

    pub fn init(raft_config: helpers::RaftConfig) -> Raft {
        // idealy this should be based on a cli flag to read
        // NOTE: persist_path does not implement Copy
        let mut state = NodeState::init_state(raft_config.persist_path.clone());

        debug!("[RAFT] recovered/initialized raft state!");

        let timeout = gen_rand_timeout();
        // state.assert_state();

        return Raft {
            config: raft_config,
            state,
            timeout,
        };
    }

    pub async fn ping_nodes(
        connections: Arc<Vec<SocketAddr>>,
        state: Arc<Mutex<NodeState>>,
    ) -> bool {
        for conn in connections.iter() {
            let node_state = state.lock().await;
            if let Some(resp) = ConnectionLayer::ping_node_wrapper(
                conn.to_owned(),
                format!("ping"),
                node_state.node_id,
            )
            .await
            {
                info!(
                    "[ Node {} ] Response from {}: {}",
                    node_state.node_id, conn, resp.0
                );
            } else {
                // warn!(
                //     "[ Node {} ] Failed to send ping to {}",
                //     node_state.node_id, conn
                // );
                return false;
            }
        }
        true
    }

    // NOTE:
    // return false if it fails to pass the criteria of a
    // leader election
    pub async fn start_leader_election(
        connections: Arc<Vec<SocketAddr>>,
        mut state: &mut NodeState,
    ) -> bool {
        info!("Initiating a leader election");

        let mut quorum_len: usize;
        let cluster_size = &connections.len();
        if state.log.len() % 2 == 0 {
            quorum_len = (cluster_size + 2) / 2;
        } else {
            quorum_len = (cluster_size + 1) / 2;
        }

        info!(
            "[ Leader Election ] Need for a quorum of votes: {}",
            quorum_len
        );

        let mut votes_recieved: Vec<NodeId> = Vec::with_capacity(2);

        for conn in connections.as_ref() {
            let log_size = state.log.len();
            let mut last_log_index: usize;
            if log_size == 0 {
                last_log_index = 0;
            } else {
                last_log_index = state.log.len() - 1;
            }

            let mut last_log_term: NodeTerm;
            match state.log.get(last_log_index) {
                Some(log_entry) => {
                    last_log_term = log_entry.term as u64;
                }
                None => {
                    last_log_term = 0;
                }
            }

            debug!("Sending Leader Election Request to {}", conn);

            if let Some(vote_request_response) = ConnectionLayer::request_vote_wrapper(
                conn.to_owned(),
                rpc::LeaderElectionRequest {
                    term: state.current_term,
                    candidate_id: state.node_id,
                    last_log_index,
                    last_log_term,
                    node_id: state.node_id,
                },
            )
            .await
            {
                if !vote_request_response.vote_granted {
                    state.node_state = State::FOLLOWER;
                    state.current_term = vote_request_response.term;
                    state.voted_for = None;
                    return false;
                } else {
                    votes_recieved.push(state.node_id);
                    debug!(
                        "Recieved vote from [ Node {} ] | Total Votes {}/{}",
                        vote_request_response.node_id,
                        votes_recieved.len(),
                        quorum_len
                    );
                    if votes_recieved.len() == quorum_len {
                        // NOTE:
                        // here we can progress to a leader
                        let _res = state.transition_to_leader();
                        info!("[ Node {} ] Transitioned to State::Leader", state.node_id);
                    }
                }
            }
        }
        false
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
    pub fn gen_rand_timeout() -> u64 {
        let mut rng = rand::thread_rng();
        // let val: u64 = rng.gen_range(200..=500);
        let val: u64 = rng.gen_range(2..4);
        val
    }
}

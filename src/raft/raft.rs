/*
    Reference:
    In search of an understandable consensus algorithm
    https://raft.github.io/raft.pdf
*/

use core::time;
use std::borrow::Borrow;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use raft::conn::ConnectionLayer;
use raft::state::NodeId;
use raft_helpers::gen_rand_timeout;
use rpc::RaftClient;
use state::NodeState;
use tarpc::tokio_serde::formats::Json;
use tarpc::{client, context};
use tokio::sync::{self, mpsc};

use crate::raft::*;
use crate::utils::helpers;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Raft {
    state: NodeState,
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
        // -

        let (mut timeout_tx, mut timeout_rx) = tokio::sync::mpsc::channel(128);
        let (mut connlayer_tx, mut connlayer_rx): (
            sync::mpsc::Sender<ConnectionLayer>,
            sync::mpsc::Receiver<ConnectionLayer>,
        ) = tokio::sync::mpsc::channel(128);

        let timeout = Arc::new(Mutex::new(self.timeout));
        let addr = Arc::new(Mutex::new(self.config.clone().listener_addr));
        let conn_sock_addrs = Arc::new(self.config.clone().connections);

        let node_state = Arc::new(Mutex::new(self.state.clone()));

        let addr_mutex = addr.clone();

        let state_clone = node_state.clone();

        // NOTE: RPC Listener
        tokio::spawn(async move {
            let node_addr = addr_mutex.lock().unwrap().clone();
            let state = Arc::clone(&state_clone);
            let mut conn = ConnectionLayer::init_layer(&node_addr, state)
                .await
                .expect("couldnt initalize a Raft Service");
            connlayer_tx.send(conn).await.unwrap();
        });

        let timeout_mutex = timeout.clone();
        let connections = conn_sock_addrs.clone();
        let state_clone = node_state.clone();

        // NOTE: timeout service
        // This spawned
        tokio::spawn(async move {
            let tx: Arc<mpsc::Sender<String>> = Arc::new(timeout_tx);
            let rpc_timeout = timeout_mutex.lock().unwrap().clone();
            let state = Arc::clone(&state_clone);

            loop {
                tokio::time::sleep(time::Duration::from_secs(rpc_timeout)).await;
                let txclone = Arc::clone(&tx);

                let socket_addrs: &Vec<SocketAddr> = connections.borrow();
                for addr in socket_addrs {
                    let mut transport = tarpc::serde_transport::tcp::connect(addr, Json::default);
                    transport.config_mut().max_frame_length(usize::MAX);

                    match transport.await {
                        Ok(transp) => {
                            let client = RaftClient::new(client::Config::default(), transp).spawn();
                            let cur_state = state.lock().unwrap().clone();
                            let resp: (String, NodeId) = client
                                .ping(context::current(), cur_state.node_id, "ping".to_owned())
                                .await
                                .unwrap_or(("failed".to_owned(), 0));
                            println!("[ Node {} ] Response from {}: {}", resp.1, addr, resp.0);
                        }
                        Err(_) => {}
                    }
                }
            }
        });

        loop {
            tokio::select! {
                Some(_timeout_msg) = timeout_rx.recv() => {
                    // info!("{timeout_msg}");
                }
                Some(_conn) = connlayer_rx.recv() => {
                    // info!(connection = %conn);
                }
            }
        }
    }

    pub fn init(raft_config: helpers::RaftConfig) -> Raft {
        // idealy this should be based on a cli flag to read
        // NOTE: persist_path does not implement Copy
        let mut state = NodeState::init_state(raft_config.persist_path.clone());

        println!("[RAFT] recovered/initialized raft state!");

        let timeout = gen_rand_timeout();
        state.assert_state();

        return Raft {
            config: raft_config,
            state,
            timeout,
        };
    }

    pub async fn start_leader_election(
        connections: Arc<Vec<SocketAddr>>,
        state: Arc<Mutex<NodeState>>,
    ) -> bool {
        let votes_recieved: Vec<NodeId> = Vec::with_capacity(2);

        for conn in connections.as_ref() {

            let binding = state.lock();
            let cur_state = binding.as_ref().unwrap();

            let vote_request_response = ConnectionLayer::request_vote_shim(
                conn.to_owned(),
                rpc::LeaderElectionRequest {
                    term: cur_state.current_term.clone(),
                    candidate_id: cur_state.node_id.clone(),
                    last_log_index: cur_state.log.len() - 1,
                    last_log_term: cur_state
                        .log
                        .get(cur_state.log.len() - 1 as usize)
                        .unwrap()
                        .term as u64,
                    node_id: cur_state.node_id,
                },
            )
            .await;
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

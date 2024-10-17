/*
    Reference:
    In search of an understandable consensus algorithm
    https://raft.github.io/raft.pdf
*/

use core::time;
use std::sync::{Arc, Mutex};
use std::thread;

use raft::conn::ConnectionLayer;
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

#[allow(unused_mut, dead_code)]
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

        // NOTE:
        // Starting a TcpListiner on the node
        //
        // A follower will be waiting

        let timeout = Arc::new(Mutex::new(self.timeout));
        let addr = Arc::new(Mutex::new(self.config.listener_addr));

        let addr_mutex = addr.clone();
        // NOTE: RPC Listene
        tokio::spawn(async move {
            let node_addr = addr_mutex.lock().unwrap().clone();
            let mut conn = ConnectionLayer::init_layer(&node_addr)
                .await
                .expect("couldnt initalize a Raft Service");
            connlayer_tx.send(conn).await.unwrap();
        });

        let timeout_mutex = timeout.clone();

        // NOTE: timeout service
        tokio::spawn(async move {
            let tx: Arc<mpsc::Sender<String>> = Arc::new(timeout_tx);
            let rpc_timeout = timeout_mutex.lock().unwrap().clone();
            loop {
                tokio::time::sleep(time::Duration::from_millis(rpc_timeout)).await;
                let txclone = Arc::clone(&tx);
                let _transmitter = Arc::into_inner(txclone);
                // transmitter.send("timeout".to_owned()).unwrap();
                println!("[TIMER SERVICE] Timeout {}!", rpc_timeout);
            }
        });

        // second block

        loop {
            tokio::select! {
                Some(timeout_msg) = timeout_rx.recv() => {
                    println!("{timeout_msg}");
                }
                Some(conn) = connlayer_rx.recv() => {
                    dbg!(&conn);
                }
            }
        }
        // tokio::select! {
        //     val = timeout_rx => {
        //         println!("Timeout for rpc: {:?}", val);
        //     }
        //     val = connlayer_rx => {
        //         println!("{:?}", val);
        //     }
        // }
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

    pub async fn heart_beats(&mut self) {
        // Abstraction over connection to send
        // heartbeats to its connections

        // select some rand timeout interaval

        let timeout = gen_rand_timeout();
        println!("timeout: {timeout:?}");

        let connections = self.config.clone().connections;

        loop {
            for conn in &connections {
                let mut transport = tarpc::serde_transport::tcp::connect(conn, Json::default);
                transport.config_mut().max_frame_length(usize::MAX);
                let transport_unwraped = transport.await;
                println!("transport");
                match transport_unwraped {
                    Ok(trans) => {
                        let raft_client = RaftClient::new(client::Config::default(), trans).spawn();
                        println!("Connection with {conn}");

                        let resp = raft_client
                            .ping(context::current(), "ping".to_owned())
                            .await
                            .unwrap();
                        println!("{resp}");
                    }
                    Err(_) => {}
                }
            }

            thread::sleep(time::Duration::from_secs(timeout));
        }
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
        let val: u64 = rng.gen_range(200..=500);
        // let val: u64 = rng.gen_range(2..3);
        val
    }
}

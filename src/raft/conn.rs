use std::future;
use std::io;
//
// TCP layer for underlying consensus library
//
// SEP 27th - flatbuffers or protobuf for rpc?
// OCT 11th - tarpc :3
//
use std::net::SocketAddr;
use std::sync::Arc;

use futures::Future;
use futures::StreamExt;
use tarpc::client;
use tarpc::context;
use tarpc::server::{self, incoming::Incoming, Channel};
use tarpc::tokio_serde::formats::Json;
use tokio;
use tokio::sync::Mutex;

use super::rpc::Raft;
use super::state::NodeId;
use super::state::NodeState;
use crate::RaftClient;
use crate::RaftServer;

type ConnErrResult<T> = Result<T, ConnErrors>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ConnErrors {
    TCPTokioError(io::Error),
    RpcError(tarpc::client::RpcError),
}

impl From<io::Error> for ConnErrors {
    fn from(value: io::Error) -> Self {
        ConnErrors::TCPTokioError(value)
    }
}

impl From<tarpc::client::RpcError> for ConnErrors {
    fn from(value: tarpc::client::RpcError) -> Self {
        ConnErrors::RpcError(value)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConnectionLayer {
    pub local_addr: SocketAddr,
    // TCP Listiner
    // listener: *mut TcpListener,
    // listener: <>,
    // pub client: RaftClient,
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[allow(unused)]
impl ConnectionLayer {
    pub async fn init_layer(
        addr: &SocketAddr,
        node_state: Arc<Mutex<NodeState>>,
    ) -> ConnErrResult<ConnectionLayer> {
        println!("[RAFT][INIT_LAYER] Starting tcp listening layer");

        dbg!("node state in timeout service {:#?}", &node_state);

        let state_clone = Arc::clone(&node_state);

        let mut listener = tarpc::serde_transport::tcp::listen(addr, Json::default).await?;
        listener.config_mut().max_frame_length(usize::MAX);
        println!("[RAFT][LISTENER] Listening {:?}", addr);

        tokio::spawn(async move {
            listener
                .filter_map(|r| future::ready(r.ok()))
                .map(server::BaseChannel::with_defaults)
                .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
                .map(|channel| {
                    let server = RaftServer {
                        node_state: Arc::clone(&state_clone),
                    };
                    channel.execute(server.serve()).for_each(spawn)
                })
                .buffer_unordered(10)
                .for_each(|_| async {})
                .await;
        });

        Ok(ConnectionLayer {
            local_addr: addr.clone(),
        })
    }

    pub async fn ping_node_wrapper(
        sock_addr: SocketAddr,
        request: String,
        node_id: NodeId,
    ) -> Option<(String, NodeId)> {
        let mut transport = tarpc::serde_transport::tcp::connect(sock_addr, Json::default);
        transport.config_mut().max_frame_length(usize::MAX);

        // println!("[DEBUG] Have a lock on state and opened a trasnporter");

        match transport.await {
            Ok(transp) => {
                let client = RaftClient::new(client::Config::default(), transp).spawn();
                let resp = client
                    .ping(context::current(), node_id, format!("ping"))
                    .await
                    .expect(&format!("Failed to send ping RPC to {}", sock_addr));
                return Some(resp);
            }
            Err(_) => None,
        }
    }

    // pub async fn request_vote_wrapper(
    //     sock_addr: SocketAddr,
    //     request: LeaderElectionRequest,
    // ) -> Option<LeaderElectionResponse> {
    //     let mut transport = tarpc::serde_transport::tcp::connect(sock_addr, Json::default);
    //     transport.config_mut().max_frame_length(usize::MAX);

    //     match transport.await {
    //         Ok(trans) => {
    //             let client = RaftClient::new(client::Config::default(), trans).spawn();

    //             let vote_request_response = client
    //                 .leader_election(context::current(), request)
    //                 .await
    //                 .expect(&format!(
    //                     "Failed to send leader_election RPC to {}",
    //                     sock_addr
    //                 ));

    //             return Some(vote_request_response);
    //         }
    //         Err(_) => {
    //             return None;
    //         }
    //     }
    // }
}

// pub async fn ping_node_wrapper(
//     sock_addr: SocketAddr,
//     request: String,
//     node_state: Arc<Mutex<NodeState>>,
// ) -> Option<(String, NodeId)> {
//     let mut transport = tarpc::serde_transport::tcp::connect(sock_addr, Json::default);
//     transport.config_mut().max_frame_length(usize::MAX);

//
// let handle = node_state.lock(); <-- this is a MutexGuard{{error}, NodeState}
//                                     and this does not a `Send`
//
// // while having this lock i try accessing the node_id from this MutexGuard
// let node_id: u64 = handle.node_id;
//
// // while having a lock we await on the tcp transporter
// match transport.await {
//                 ^^^^^
//                 why is this a problem now?
//
// Consider a thread where the current function is being executed
// where we have a lock
//     T1 ────[LOCK]────[AWAIT] ─────────────
//                      ^^^^^^^
//                 here an await is
//                 called while we have a lock
//
//     tokio handles how it resumens execution, after the future
//     is done executing, this could be on _another thread_?
//
//     T1 ────[LOCK]────[AWAIT] ─────────────
//               │
//     T2        │                   ─────[LOCK]───
//               │                            │
//               └─────────────────┘
//                 transfering locked
//                mutex across threads
// }

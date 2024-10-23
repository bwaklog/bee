use std::sync::Arc;

use futures::Future;
use serde::{Deserialize, Serialize};
use tarpc;
use tokio::sync::Mutex;
use tracing::info;

use crate::store;

use super::state::{LogIndex, NodeId, NodeState, NodeTerm};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppendEntriesRequest {
    pub term: NodeTerm,
    pub leader_id: NodeId,
    pub prev_log_entry: LogIndex,
    pub prev_log_term: NodeTerm,

    pub entries: Vec<store::LogEntry>,
    pub leader_commit_index: LogIndex,

    // Extra information
    pub node_id: NodeId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppendEntriesResponse {
    pub term: NodeTerm,
    pub success: bool,

    // Extra information
    pub node_id: NodeId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeaderElectionRequest {
    pub term: NodeTerm,
    pub candidate_id: NodeId,
    pub last_log_index: LogIndex,
    pub last_log_term: NodeTerm,

    // Extra information
    pub node_id: NodeId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeaderElectionResponse {
    pub term: NodeTerm,
    pub vote_granted: bool,

    // extra information
    pub node_id: NodeId,
}

#[tarpc::service]
pub trait Raft {
    // async fn ping(node_id: u64, ping: String) -> (String, NodeId);
    async fn ping(node_id: u64, ping: String) -> (String, NodeId);

    async fn echo(input: String) -> String;

    // async fn append_entries(request: AppendEntriesRequest) -> AppendEntriesResponse;
    async fn leader_election(request: LeaderElectionRequest) -> LeaderElectionResponse;
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct RaftServer {
    pub node_state: Arc<Mutex<NodeState>>,
}

#[allow(unused)]
async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

impl Raft for RaftServer {
    async fn echo(self, _: tarpc::context::Context, input: String) -> String {
        // let node_state = Arc::clone(&self.node_state);
        // let safe_state = node_state.lock().await;
        // safe_state.echo(input).await
        format!("{input}")
    }

    async fn ping(
        self,
        _: tarpc::context::Context,
        node_id: u64,
        ping: String,
    ) -> (String, NodeId) {
        let server = self.node_state.lock().await;
        let current_state = server.get_state();
        if ping == "ping" {
            info!(
                "[RPC][ Node {} ] Recieved ping from {}. Current State {:?}",
                server.node_id, node_id, current_state
            );
            return (format!("pong"), node_id);
        } else {
            return ("nope".to_owned(), 0);
        }
    }

    // #[allow(unused)]
    // fn ping(self, context: tarpc::context::Context, node_id: u64, ping: String) -> Self::PingFut {
    //     // let server = self.node_state.lock().unwrap();
    //     // if ping == "ping" {
    //     //     println!(
    //     //         "[ Node {} ] Recieved ping from {}. Current state {:?}",
    //     //         server.node_id,
    //     //         node_id,
    //     //         server.get_state(),
    //     //     );
    //     //     future::ready((format!("pong"), server.node_id))
    //     // } else {
    //     //     future::ready((format!("why :("), server.node_id))
    //     // }
    //     future::ready((format!("nope"), 0))
    // }

    // #[allow(unused)]
    // fn append_entries(
    //     self,
    //     context: tarpc::context::Context,
    //     request: AppendEntriesRequest,
    // ) -> Self::AppendEntriesFut {
    //     // TODO
    //     future::ready(AppendEntriesResponse {
    //         node_id: 0,
    //         term: 0,
    //         success: false,
    //     })
    // }

    // NOTE: Options and Results not handled
    #[allow(unused)]
    async fn leader_election(
        self,
        context: tarpc::context::Context,
        request: LeaderElectionRequest,
    ) -> LeaderElectionResponse {
        info!(
            "Recieved leader vote request from [ Node {} ]",
            request.node_id
        );

        let mut state_lock = self.node_state.lock().await;

        let node_details = state_lock.get_state().unwrap();

        if node_details.0 > request.term {
            return LeaderElectionResponse {
                term: node_details.0,
                vote_granted: false,
                node_id: node_details.2,
            };
        }

        // NOTE: this should be usize -> considering log lengths
        let mut last_term: u64 = 0;
        if state_lock.log.len() > 0 {
            last_term = state_lock.log.len() as u64 - 1;
        }

        // checking if the log is ok!
        let log_ok: bool = (request.last_log_term > last_term)
            || (state_lock.current_term == request.term
                && request.last_log_index + 1 >= state_lock.log.len());

        let mut voted_for: NodeId = 0;

        // evaluate the condition
        // if cTerm == currentTerm ^ logOk ^ votedFor belongs to {cNodeId, null}
        if request.term == state_lock.current_term
            || log_ok
            || (state_lock.voted_for == None || state_lock.voted_for == Some(request.node_id))
        {
            state_lock.voted_for = Some(request.node_id);
            info!("Voted for [ Node {} ]", request.node_id);
            return LeaderElectionResponse {
                term: state_lock.current_term,
                vote_granted: true,
                node_id: state_lock.node_id,
            };
        } else {
            info!("Rejected voted for [ Node {} ]", request.node_id);
            return LeaderElectionResponse {
                term: state_lock.current_term,
                vote_granted: false,
                node_id: state_lock.node_id,
            };
        }
    }
}

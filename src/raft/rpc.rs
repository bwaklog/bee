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
    // async fn leader_election(request: LeaderElectionRequest) -> LeaderElectionResponse;
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

    // // NOTE: Options and Results not handled
    // #[allow(unused)]
    // fn leader_election(
    //     self,
    //     context: tarpc::context::Context,
    //     request: LeaderElectionRequest,
    // ) -> Self::LeaderElectionFut {
    //     future::ready(LeaderElectionResponse {
    //         term: 0,
    //         vote_granted: false,
    //         node_id: 0,
    //     })

    //     // let mut state_lock = self.node_state.lock();

    //     // let state_mut = state_lock.as_mut().unwrap();
    //     // let node_details = state_mut.get_state().unwrap();

    //     // if node_details.0 > request.term {
    //     //     future::ready(LeaderElectionResponse {
    //     //         term: node_details.0,
    //     //         vote_granted: false,
    //     //         node_id: state_mut.node_id,
    //     //     });
    //     // }

    //     // // NOTE: this should be usize -> considering log lengths
    //     // let mut last_term: u64 = 0;
    //     // if state_mut.log.len() > 0 {
    //     //     last_term = state_mut.log.len() as u64 - 1;
    //     // }

    //     // // checking if the log is ok!
    //     // let log_ok: bool = (request.last_log_term > last_term)
    //     //     || (state_mut.current_term == request.term
    //     //         && request.last_log_index + 1 >= state_mut.log.len());

    //     // let mut voted_for: NodeId = 0;

    //     // // evaluate the condition
    //     // // if cTerm == currentTerm ^ logOk ^ votedFor belongs to {cNodeId, null}
    //     // if request.term == state_mut.current_term
    //     //     || log_ok
    //     //     || vec![request.node_id, 0].contains(&state_mut.voted_for.unwrap())
    //     // {
    //     //     voted_for = request.node_id;
    //     //     future::ready(LeaderElectionResponse {
    //     //         term: state_mut.current_term,
    //     //         vote_granted: true,
    //     //         node_id: state_mut.node_id,
    //     //     })
    //     // } else {
    //     //     future::ready(LeaderElectionResponse {
    //     //         term: state_mut.current_term,
    //     //         vote_granted: false,
    //     //         node_id: state_mut.node_id,
    //     //     })
    //     // }
    // }
}

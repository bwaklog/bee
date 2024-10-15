use futures::future::{self, Ready};
use serde::{Deserialize, Serialize};
use tarpc;

use crate::store;

use super::state::{LogIndex, NodeId, NodeTerm};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppendEntriesRequest {
    pub term: NodeTerm,
    pub leader_id: NodeId,
    pub prev_log_entry: LogIndex,
    pub prev_log_term: NodeTerm,

    pub entries: Vec<store::LogEntry>,
    pub leader_commit_index: LogIndex,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppendEntriesResponse {
    pub term: NodeTerm,
    pub success: bool,
}

#[tarpc::service]
pub trait Raft {
    async fn ping(ping: String) -> String;

    async fn append_entries(request: AppendEntriesRequest) -> AppendEntriesResponse;
}

#[derive(Clone, Debug)]
pub struct RaftServer;

impl Raft for RaftServer {
    type PingFut = Ready<String>;
    type AppendEntriesFut = Ready<AppendEntriesResponse>;

    #[allow(unused)]
    fn ping(self, context: tarpc::context::Context, ping: String) -> Self::PingFut {
        if ping == "ping" {
            future::ready(format!("ping UWU"))
        } else {
            future::ready(format!("why :("))
        }
    }

    #[allow(unused)]
    fn append_entries(
        self,
        context: tarpc::context::Context,
        request: AppendEntriesRequest,
    ) -> Self::AppendEntriesFut {
        // TODO
        future::ready(AppendEntriesResponse {
            term: 0,
            success: false,
        })
    }
}

// async fn server_main() {

// }

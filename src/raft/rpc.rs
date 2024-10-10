use futures::future::{self, Ready};
use tarpc;

#[tarpc::service]
pub trait Raft {
    async fn ping(ping: String) -> String;
}

#[derive(Clone, Debug)]
pub struct RaftServer;

impl Raft for RaftServer {
    type PingFut = Ready<String>;

    fn ping(self, context: tarpc::context::Context, ping: String) -> Self::PingFut {
        if ping == "ping" {
            future::ready(format!("ping UWU"))
        } else {
            future::ready(format!("why :("))
        }
    }
}

// async fn server_main() {

// }

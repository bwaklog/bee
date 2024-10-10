//
// TCP layer for underlying consensus library
//
// SEP 27th - flatbuffers or protobuf for rpc?
// OCT 11th - tarpc :3
//
use std::net::SocketAddr;
use std::net::TcpListener;

use tarpc::client;
use tarpc::client::RpcError;
use tarpc::context;
use tarpc::server;
use tarpc::server::Channel;

use crate::RaftClient;
use crate::RaftServer;

use super::rpc::Raft;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ConnectionLayer {
    local_addr: SocketAddr,
    listener: *mut TcpListener,

    pub client: RaftClient,
}

#[allow(unused)]
impl ConnectionLayer {
    pub fn init_layer(addr: &SocketAddr) -> ConnectionLayer {
        let mut listener = TcpListener::bind(addr).unwrap();

        let (client_transporter, server_transporter) = tarpc::transport::channel::unbounded();
        let server = server::BaseChannel::with_defaults(server_transporter);
        let raft_server = server.as_ref();

        tokio::spawn(server.execute(RaftServer.serve()));

        let client = RaftClient::new(client::Config::default(), client_transporter).spawn();

        return ConnectionLayer {
            local_addr: addr.clone(),
            listener: &mut listener,
            client,
        };
    }

    pub async fn ping(self, msg: String) -> Result<String, RpcError> {
        let ping = self.client.ping(context::current(), msg.to_string()).await;
        ping
    }
}

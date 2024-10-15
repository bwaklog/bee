use std::io;
use std::future;
//
// TCP layer for underlying consensus library
//
// SEP 27th - flatbuffers or protobuf for rpc?
// OCT 11th - tarpc :3
//
use std::net::SocketAddr;

use futures::StreamExt;
use tarpc::server::incoming::Incoming;
use tarpc::server::Channel;
use tarpc::server::{self};
use tokio;
use tarpc::{client, context};

use tarpc::tokio_serde::formats::Json;

use crate::{RaftClient, RaftServer};

use super::rpc::Raft;

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

#[allow(unused)]
impl ConnectionLayer {
    pub async fn init_layer(addr: &SocketAddr) -> ConnErrResult<ConnectionLayer> {
        println!("[RAFT][INIT_LAYER] Starting tcp listening layer");

        let mut listener = tarpc::serde_transport::tcp::listen(addr, Json::default).await?;
        listener.config_mut().max_frame_length(usize::MAX);
        println!("[RAFT][LISTENER] Listening {:?}", addr);

        // listener = TcpListener
        //
        // loop {
        //   // listen on condition
        // }

        tokio::spawn(async {
            listener
                .filter_map(|r| future::ready(r.ok()))
                .map(server::BaseChannel::with_defaults)
                .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
                .map(|channel| {
                    println!("[RAFT][LISTINER] something happened!");
                    let server = RaftServer;
                    channel.execute(server.serve())
                })
                .buffer_unordered(10)
                .for_each(|_| async {})
                .await;
        });

        Ok(ConnectionLayer {
            local_addr: addr.clone(),
        })
    }

    pub async fn ping(addr: &SocketAddr, ping: String) -> ConnErrResult<String> {
        // println!("[RAFT][CLIENT] Sending ping to {addr}");

        let mut transport = tarpc::serde_transport::tcp::connect(addr, Json::default);
        transport.config_mut().max_frame_length(usize::MAX);

        let raft_client = RaftClient::new(client::Config::default(), transport.await?).spawn();
        println!("Connection with {addr}");

        let resp = raft_client.ping(context::current(), ping).await?;
        println!("{resp}");

        Ok(resp)
    }
}

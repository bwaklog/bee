mod raft;
mod storage;
mod utils;

use clap::Parser;
use raft::raft::Raft;
use core::time;
use std::{error::Error, path::PathBuf};
use tokio::signal;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    conf_path: String,
}

use raft::rpc::*;
use storage::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config = utils::helpers::parse_config(PathBuf::from(args.conf_path)).unwrap();
    // dbg!(&config);

    // compile prtobuf files

    println!("\nInitializing raft layer:");
    let mut raft: Raft = Raft::init(config.raft.clone());

    println!("[MAIN] Starting node daemon");

    tokio::spawn(async move {
        raft.node_daemon().await;
    });


    // println!(
    //     "[MAIN] Finished initializing raft client: {}",
    //     config.node_name
    // );

    // tokio::spawn(async move { raft_node.heart_beats().await });
    // raft_node.heart_beats().await;

    // raft_node.heart_beats().await;

    // match signal::ctrl_c().await {
    //     Ok(()) => {}
    //     Err(e) => eprintln!("Unable to listen for shutdown: {:?}", e),
    // }

    Ok(())
}

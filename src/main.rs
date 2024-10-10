mod raft;
mod storage;
mod utils;

use clap::Parser;
use raft::raft::Raft;
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    conf_path: String,
}

use storage::*;
use raft::rpc::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config = utils::helpers::parse_config(PathBuf::from(args.conf_path)).unwrap();
    // dbg!(&config);

    // compile prtobuf files

    println!("\nInitializing raft layer:");
    let raft = Raft::init(config.raft.clone());

    let ping = raft.conn.ping("ping".to_string()).await?;

    println!("{ping}");



    Ok(())
}

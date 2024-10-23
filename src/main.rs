mod raft;
mod storage;
mod utils;

use clap::Parser;
use raft::raft::Raft;
use std::{error::Error, path::PathBuf};
use tokio::signal;
use tracing::{debug, level_filters::LevelFilter, warn};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    conf_path: String,
}

use raft::rpc::*;
use storage::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = filter::filter_fn(|metadata| metadata.target().starts_with("bee"));

    tracing_subscriber::registry()
        .with(filter)
        // .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .with(tracing_subscriber::fmt::layer().with_filter(LevelFilter::DEBUG))
        .init();

    let args = Args::parse();

    let config = utils::helpers::parse_config(PathBuf::from(args.conf_path)).unwrap();
    // dbg!(&config);

    debug!("initializing consensus layer");
    let mut raft: Raft = Raft::init(config.raft.clone());

    debug!("initializing node daemon");
    tokio::spawn(async move {
        raft.node_daemon().await;
    });

    debug!(
        "[MAIN] Finished initializing raft client: {}",
        config.node_name
    );

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(e) => {
            warn!("Unable to listen for shutdown: {:?}", e)
        }
    }

    Ok(())
}

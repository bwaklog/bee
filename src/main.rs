mod raft;
mod storage;
mod utils;

use clap::Parser;
use raft::raft::Raft;
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
    // let subscriber = FmtSubscriber::builder()
    //     .with_max_level(tracing::Level::TRACE)
    //     .with_timer(time::ChronoLocal::rfc_3339())
    //     .with_target(true)
    //     .with_writer(io::stderr)
    //     .with_file(true)
    //     .with_line_number(true)
    //     .finish();
    //
    // tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let config = utils::helpers::parse_config(PathBuf::from(args.conf_path)).unwrap();
    // dbg!(&config);

    // compile prtobuf files

    println!("initializing consensus layer");
    let mut raft: Raft = Raft::init(config.raft.clone());

    println!("initializing node daemon");
    tokio::spawn(async move {
        raft.node_daemon().await;
    });

    println!(
        "[MAIN] Finished initializing raft client: {}",
        config.node_name
    );

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(e) => {
            println!("Unable to listen for shutdown: {:?}", e)
        }
    }

    Ok(())
}

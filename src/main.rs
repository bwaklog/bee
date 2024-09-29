mod raft;
mod storage;
mod utils;

use std::error::Error;
use utils::helpers;

use storage::*;

fn main() -> Result<(), Box<dyn Error>> {
    let config = utils::helpers::parse_config().unwrap();
    dbg!(config);
    Ok(())
}

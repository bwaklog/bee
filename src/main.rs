mod storage;
mod raft;

use storage::*;
use raft::raft::*;

fn main() {
    println!("Raft and kv's");
    // let raft: Raft<u32> = Raft::init();
    // dbg!(raft);
}

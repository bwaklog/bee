// Server-Client based architecture for the kv,
// we just have a TCP Listiner for a client to
// connect to and send requests
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Operation {
    SET,
    GET,
    DEL,
}

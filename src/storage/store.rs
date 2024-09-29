// Serializing and Deserializing methods
// for data
//
// data serialization before writes to disk
// and network communication
//
use crate::storage::kv;
use rmp;
use serde::{Deserialize, Serialize};

use super::kv::Operation;

#[allow(dead_code)]
pub struct MemKV {}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Data {
    INT(u32),
    STRING(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    operation: kv::Operation,
    term: u32,
    // need not be a u32, but
    // im neglecting this for now
    key: u32,
    data: Data,
}

impl LogEntry {
    // test functions
    pub fn new_entry(operation: Operation, key: u32, data: Data, term: u32) -> LogEntry {
        return LogEntry {
            operation,
            term,
            key,
            data,
        };
    }
}

impl PartialEq for LogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.data == other.data && self.operation == other.operation
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::kv::Operation;

    use super::{Data, LogEntry};

    #[test]
    fn serialize() {
        let entry = LogEntry::new_entry(Operation::SET, 2, Data::INT(2), 0);
        let ser_entry = serde_json::to_string(&entry).unwrap();
        let mut buff = Vec::new();
        rmp::encode::write_str(&mut buff, &ser_entry).unwrap();
        println!("serde: {}", ser_entry);
        println!("msgpack: {:?}", buff);
    }
}

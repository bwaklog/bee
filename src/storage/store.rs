// Serializing and Deserializing methods
// for data
//
// data serialization before writes to disk
// and network communication
//
use super::kv::Operation;
use crate::storage::kv;
use serde::{Deserialize, Serialize};
// use rmp_serde;

#[allow(dead_code)]
pub struct MemKV {}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Data {
    INT(u32),
    STRING(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub operation: kv::Operation,
    pub term: u32,
    // need not be a u32, but
    // im neglecting this for now
    pub key: u32,
    pub data: Data,
}

impl LogEntry {
    // test functions
    #[allow(unused)]
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

    // #[ignore = "testing rmp_serde"]
    #[test]
    fn serialize() {
        let entry = LogEntry::new_entry(Operation::SET, 2, Data::INT(2), 0);
        let entry2 = LogEntry::new_entry(Operation::SET, 2, Data::INT(2), 0);
        let mut logs = Vec::new();
        logs.push(entry);
        logs.push(entry2);
        let bytes = rmp_serde::to_vec(&logs).unwrap();
        println!("rmp_serde: {:?}", bytes);
    }
}

use crate::storage::kv;

use super::kv::Operation;

#[allow(dead_code)]
pub struct MemKV {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Data {
    INT(u32),
    STRING(String),
}

#[derive(Debug, Clone)]
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

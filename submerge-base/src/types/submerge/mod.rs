use crate::types::substrate::block_trace::StorageMethod;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct BlockTrace {
    pub index: u32,
    pub key: String,
    pub value: String,
    pub ext_id: String,
    pub method: StorageMethod,
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockTraces {
    pub block_hash: String,
    pub block_parent_hash: String,
    pub block_number: u64,
    pub spec_version: u32,
    pub is_finalized: bool,
    pub traces: Vec<BlockTrace>,
}

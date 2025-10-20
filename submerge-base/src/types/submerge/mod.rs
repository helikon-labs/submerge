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

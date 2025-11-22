use serde::{Deserialize, Serialize};

use crate::types::{api::dto::pagination::PaginationQuery, persistence::TraceRow};

#[derive(Clone, Debug, Serialize)]
pub enum TraceType {
    StorageItem,
    KnownKey,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub min_block_number: Option<u64>,
    pub max_block_number: Option<u64>,
    pub min_block_timestamp: Option<u64>,
    pub max_block_timestamp: Option<u64>,
    pub min_spec_version: Option<u32>,
    pub max_spec_version: Option<u32>,
    pub key_prefix: Option<String>,
    pub key_params: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceDTO {
    pub block_hash: String,
    pub block_number: u64,
    pub spec_version: u32,
    pub index: u32,
    pub key_prefix: String,
    pub key_params: Option<String>,
    pub value: Option<String>,
    pub ext_id: String,
    pub method: String,
    pub parent_id: Option<String>,
    #[serde(rename = "type")]
    pub trace_type: Option<TraceType>,
}

impl From<&TraceRow> for TraceDTO {
    fn from(row: &TraceRow) -> Self {
        Self {
            block_hash: format!("0x{}", hex::encode(&row.block_hash)),
            block_number: row.block_number as u64,
            spec_version: row.spec_version as u32,
            index: row.index as u32,
            key_prefix: format!("0x{}", hex::encode(&row.key_prefix)),
            key_params: row
                .key_params
                .as_ref()
                .map(|key_params| format!("0x{}", hex::encode(key_params))),
            value: row
                .value
                .as_ref()
                .map(|value| format!("0x{}", hex::encode(value))),
            ext_id: format!("0x{}", hex::encode(&row.ext_id)),
            method: row.method.clone(),
            parent_id: row.parent_id.clone(),
            trace_type: if row.metadata_storage_item_id.is_some() {
                Some(TraceType::StorageItem)
            } else if row.is_known_key {
                Some(TraceType::KnownKey)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockTraceQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

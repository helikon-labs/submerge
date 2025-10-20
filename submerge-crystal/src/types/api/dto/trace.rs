use serde::{Deserialize, Serialize};

use crate::types::{api::dto::pagination::PaginationQuery, persistence::TraceRow};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub key: Option<String>,
    pub key_prefix: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceDTO {
    pub block_hash: String,
    pub block_number: u64,
    pub spec_version: u32,
    pub index: u32,
    pub key: String,
    pub key_prefix: String,
    pub value: Option<String>,
    pub ext_id: String,
    pub method: String,
    pub parent_id: Option<String>,
}

impl From<&TraceRow> for TraceDTO {
    fn from(row: &TraceRow) -> Self {
        Self {
            block_hash: format!("0x{}", hex::encode(&row.block_hash)),
            block_number: row.block_number as u64,
            spec_version: row.spec_version as u32,
            index: row.index as u32,
            key: format!("0x{}", hex::encode(&row.key)),
            key_prefix: format!("0x{}", hex::encode(&row.key_prefix)),
            value: row
                .value
                .as_ref()
                .map(|value| format!("0x{}", hex::encode(value))),
            ext_id: format!("0x{}", hex::encode(&row.ext_id)),
            method: row.method.clone(),
            parent_id: row.parent_id.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockTraceQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

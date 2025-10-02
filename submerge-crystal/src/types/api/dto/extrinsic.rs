use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;

use crate::types::{api::dto::pagination::PaginationQuery, persistence::ExtrinsicRow, BlockStatus};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtrinsicQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub min_block_number: Option<u64>,
    pub max_block_number: Option<u64>,
    pub min_block_timestamp: Option<u64>,
    pub max_block_timestamp: Option<u64>,
    pub min_spec_version: Option<u64>,
    pub max_spec_version: Option<u64>,
    pub is_signed: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockExtrinsicQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub is_signed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Extrinsic {
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub spec_version: u32,
    pub block_status: BlockStatus,
    pub trace_index: Option<u32>,
    pub hash: String,
    pub index: u32,
    pub version: u32,
    pub signer: Option<String>,
    pub signature: Option<String>,
    pub extra: Option<JSONValue>,
    pub is_successful: bool,
}

impl From<&ExtrinsicRow> for Extrinsic {
    fn from(row: &ExtrinsicRow) -> Self {
        Self {
            block_hash: format!("0x{}", hex::encode(&row.block_hash)),
            block_number: row.block_number as u64,
            block_timestamp: row.block_timestamp as u64,
            spec_version: row.spec_version as u32,
            block_status: row.block_status,
            trace_index: row.trace_index.map(|i| i as u32),
            hash: format!("0x{}", hex::encode(row.hash)),
            index: row.index as u32,
            version: row.version as u32,
            signer: row
                .signer
                .as_ref()
                .map(|signer| format!("0x{}", hex::encode(signer))),
            signature: row
                .signature
                .as_ref()
                .map(|signature| format!("0x{}", hex::encode(signature))),
            extra: row.extra.clone(),
            is_successful: row.is_successful,
        }
    }
}

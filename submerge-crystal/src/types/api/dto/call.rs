use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;

use crate::types::{
    api::dto::pagination::PaginationQuery, persistence::CallCompositeRow, BlockStatus,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub min_block_number: Option<u64>,
    pub max_block_number: Option<u64>,
    pub min_block_timestamp: Option<u64>,
    pub max_block_timestamp: Option<u64>,
    pub min_spec_version: Option<u32>,
    pub max_spec_version: Option<u32>,
    pub pallet_name: Option<String>,
    pub pallet_call_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockCallQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub pallet_name: Option<String>,
    pub pallet_call_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallDTO {
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: Option<u64>,
    pub spec_version: u32,
    pub block_status: BlockStatus,
    pub extrinsic_index: u32,
    pub extrinsic_hash: String,
    pub nesting_index: Option<String>,
    pub pallet_index: u32,
    pub pallet_name: String,
    pub pallet_call_index: u32,
    pub pallet_call_name: String,
    pub is_successful: bool,
    pub args: JSONValue,
}

impl From<&CallCompositeRow> for CallDTO {
    fn from(row: &CallCompositeRow) -> Self {
        Self {
            block_hash: format!("0x{}", hex::encode(&row.block_hash)),
            block_number: row.block_number as u64,
            block_timestamp: row.block_timestamp.map(|timestamp| timestamp as u64),
            spec_version: row.spec_version as u32,
            block_status: row.block_status,
            extrinsic_index: row.extrinsic_index as u32,
            extrinsic_hash: format!("0x{}", hex::encode(row.extrinsic_hash)),
            nesting_index: row.nesting_index.clone(),
            pallet_index: row.pallet_index as u32,
            pallet_name: row.pallet_name.clone(),
            pallet_call_index: row.pallet_call_index as u32,
            pallet_call_name: row.pallet_call_name.clone(),
            is_successful: row.is_successful,
            args: row.args.clone(),
        }
    }
}

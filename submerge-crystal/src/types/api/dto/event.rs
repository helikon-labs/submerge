use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;

use crate::types::{
    api::dto::pagination::PaginationQuery, persistence::EventCompositeRow, BlockStatus,
};

/*
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub min_block_number: Option<u64>,
    pub max_block_number: Option<u64>,
    pub min_block_timestamp: Option<u64>,
    pub max_block_timestamp: Option<u64>,
    pub min_spec_version: Option<u64>,
    pub max_spec_version: Option<u64>,
    pub pallet_name: Option<String>,
    pub pallet_event_name: Option<String>,
}
*/

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockEventQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub pallet_name: Option<String>,
    pub pallet_event_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDTO {
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: Option<u64>,
    pub spec_version: u32,
    pub block_status: BlockStatus,
    pub trace_index: Option<u32>,
    pub pallet_index: u32,
    pub pallet_name: String,
    pub pallet_event_index: u32,
    pub pallet_event_name: String,
    pub extrinsic_index: Option<u32>,
    pub extrinsic_hash: Option<String>,
    pub phase: String,
    pub index: u32,
    pub args: JSONValue,
}

impl From<&EventCompositeRow> for EventDTO {
    fn from(row: &EventCompositeRow) -> Self {
        Self {
            block_hash: format!("0x{}", hex::encode(&row.block_hash)),
            block_number: row.block_number as u64,
            block_timestamp: row.block_timestamp.map(|timestamp| timestamp as u64),
            spec_version: row.spec_version as u32,
            block_status: row.block_status,
            trace_index: row.trace_index.map(|i| i as u32),
            pallet_index: row.pallet_index as u32,
            pallet_name: row.pallet_name.clone(),
            pallet_event_index: row.pallet_event_index as u32,
            pallet_event_name: row.pallet_event_name.clone(),
            extrinsic_index: row.extrinsic_index.map(|i| i as u32),
            extrinsic_hash: row
                .extrinsic_hash
                .map(|hash| format!("0x{}", hex::encode(hash))),
            phase: row.phase.clone(),
            index: row.index as u32,
            args: row.args.clone(),
        }
    }
}

use crate::types::{BlockStatus, Event, Extrinsic};
use serde_json::Value as JSONValue;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct BlockRow {
    pub hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub state_root: Vec<u8>,
    pub extrinsic_root: Vec<u8>,
    pub number: i64,
    pub timestamp: Option<i64>,
    pub spec_version: i32,
    pub status: BlockStatus,
    pub weight: Option<JSONValue>,
    pub extrinsic_count: i32,
    pub event_count: i32,
    pub author_multi_address: Option<Vec<u8>>,
}

#[derive(Debug, FromRow)]
pub struct EventRow {
    #[allow(dead_code)]
    pub id: i64,
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub block_timestamp: Option<i64>,
    pub spec_version: i32,
    pub block_status: BlockStatus,
    pub trace_index: Option<i32>,
    pub metadata_event_id: i32,
    pub extrinsic_index: Option<i32>,
    pub extrinsic_hash: Option<[u8; 32]>,
    pub phase: String,
    pub index: i32,
    pub args: JSONValue,
}

impl EventRow {
    pub fn from_block_event(
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: Option<u64>,
        spec_version: u32,
        block_status: BlockStatus,
        event: &Event,
        extrinsics: &[Extrinsic],
        metadata_event_id: u32,
    ) -> Self {
        let (phase, maybe_extrinsic) = match &event.phase {
            frame_system::Phase::ApplyExtrinsic(extrinsic_index) => {
                ("ApplyExtrinsic", extrinsics.get(*extrinsic_index as usize))
            }
            frame_system::Phase::Finalization => ("Finalization", None),
            frame_system::Phase::Initialization => ("Initialization", None),
        };
        Self {
            id: 0,
            block_hash: block_hash.into(),
            block_number: block_number as i64,
            block_timestamp: block_timestamp.map(|timestamp| timestamp as i64),
            spec_version: spec_version as i32,
            block_status,
            trace_index: event.trace_index.map(|i| i as i32),
            metadata_event_id: metadata_event_id as i32,
            extrinsic_index: maybe_extrinsic.map(|e| e.index as i32),
            extrinsic_hash: maybe_extrinsic.map(|e| e.hash),
            phase: phase.to_string(),
            index: event.index as i32,
            args: event.args.clone(),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct EventCompositeRow {
    #[allow(dead_code)]
    pub id: i64,
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub block_timestamp: Option<i64>,
    pub spec_version: i32,
    pub block_status: BlockStatus,
    pub trace_index: Option<i32>,
    pub pallet_index: i32,
    pub pallet_name: String,
    pub pallet_event_index: i32,
    pub pallet_event_name: String,
    pub extrinsic_index: Option<i32>,
    pub extrinsic_hash: Option<[u8; 32]>,
    pub phase: String,
    pub index: i32,
    pub args: JSONValue,
}

#[derive(Debug, FromRow)]
pub struct LogRow {
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub index: i32,
    #[sqlx(rename = "type")]
    pub ty: String,
    pub engine: Option<String>,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, FromRow)]
pub struct ExtrinsicRow {
    #[allow(dead_code)]
    pub id: i64,
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub block_timestamp: Option<i64>,
    pub spec_version: i32,
    pub block_status: BlockStatus,
    pub trace_index: Option<i32>,
    pub hash: [u8; 32],
    pub index: i32,
    pub version: i32,
    pub signer_multi_address: Option<Vec<u8>>,
    pub signature: Option<Vec<u8>>,
    pub extra: Option<JSONValue>,
    pub is_successful: bool,
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct CallRow {
    pub id: i64,
    pub hash: Vec<u8>,
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub block_timestamp: Option<i64>,
    pub spec_version: i32,
    pub block_status: BlockStatus,
    pub extrinsic_id: i64,
    pub extrinsic_index: i32,
    pub extrinsic_hash: [u8; 32],
    pub parent_call_id: Option<i32>,
    pub nesting_index: Option<String>,
    pub pallet_index: i32,
    pub pallet_name: String,
    pub pallet_call_index: i32,
    pub pallet_call_name: String,
    pub args: JSONValue,
    pub is_successful: bool,
}

#[derive(Debug, FromRow)]
pub struct TraceRow {
    #[allow(dead_code)]
    pub id: i64,
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub spec_version: i32,
    pub index: i32,
    pub key_prefix: Vec<u8>,
    pub key_params: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
    pub ext_id: Vec<u8>,
    pub method: String,
    pub parent_id: Option<String>,
    pub metadata_storage_item_id: Option<i32>,
    pub is_known_key: bool,
}

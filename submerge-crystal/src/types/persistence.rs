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
    pub timestamp: i64,
    pub spec_version: i32,
    pub status: BlockStatus,
    pub weight: JSONValue,
    pub extrinsic_count: i32,
    pub event_count: i32,
    pub author_account_id: Vec<u8>,
}

#[derive(Debug, FromRow)]
pub struct EventRow {
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub block_timestamp: i64,
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
    pub args_json: JSONValue,
}

impl EventRow {
    pub fn from_block_event(
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        block_status: BlockStatus,
        event: &Event,
        extrinsics: &[Extrinsic],
    ) -> Self {
        let (phase, maybe_extrinsic) = match &event.phase {
            frame_system::Phase::ApplyExtrinsic(extrinsic_index) => {
                ("ApplyExtrinsic", extrinsics.get(*extrinsic_index as usize))
            }
            frame_system::Phase::Finalization => ("Finalization", None),
            frame_system::Phase::Initialization => ("Initialization", None),
        };
        Self {
            block_hash: block_hash.into(),
            block_number: block_number as i64,
            block_timestamp: block_timestamp as i64,
            spec_version: spec_version as i32,
            block_status,
            trace_index: event.trace_index.map(|i| i as i32),
            pallet_index: event.pallet_index as i32,
            pallet_name: event.pallet_name.clone(),
            pallet_event_index: event.pallet_event_index as i32,
            pallet_event_name: event.pallet_event_name.clone(),
            extrinsic_index: maybe_extrinsic.map(|e| e.index as i32),
            extrinsic_hash: maybe_extrinsic.map(|e| e.hash),
            phase: phase.to_string(),
            index: event.index as i32,
            args_json: event.args.clone(),
        }
    }
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
    pub block_timestamp: i64,
    pub spec_version: i32,
    pub block_status: BlockStatus,
    pub trace_index: Option<i32>,
    pub hash: [u8; 32],
    pub index: i32,
    pub version: i32,
    pub signer: Option<Vec<u8>>,
    pub signature: Option<Vec<u8>>,
    pub extra: Option<JSONValue>,
    pub is_successful: bool,
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct CallRow {
    pub block_hash: Vec<u8>,
    pub block_number: i64,
    pub block_timestamp: i64,
    pub spec_version: i32,
    pub block_status: BlockStatus,
    pub extrinsic_id: i64,
    pub extrinsic_index: i32,
    pub extrinsic_hash: [u8; 32],
    pub parent_call_id: Option<i64>,
    pub nesting_index: Option<String>,
    pub pallet_index: i32,
    pub pallet_name: String,
    pub pallet_call_index: i32,
    pub pallet_call_name: String,
    pub is_successful: bool,
    pub args: JSONValue,
}

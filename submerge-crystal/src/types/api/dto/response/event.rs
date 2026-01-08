use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::{
    api::dto::{
        pagination::{CursorPaginationData, PaginationData},
        request::event::EventQuery,
        response::{
            example::event::event_example, hex::Hash256Hex, schema::event::event_args_schema,
        },
    },
    persistence::EventCompositeRow,
    BlockStatus,
};

/// A runtime event.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = Event,
    example = event_example,
)]
pub(crate) struct EventDTO {
    /// Artificial event hash.
    pub hash: Hash256Hex,
    /// Hash of the event's block.
    pub block_hash: Hash256Hex,
    /// Number of the event's block.
    #[schema(example = 3172595)]
    pub block_number: u64,
    /// Timestamp of the event's block. Milliseconds.
    #[schema(required = false, nullable = false, example = 1755773684012u64)]
    pub block_timestamp: Option<u64>,
    /// Runtime spec version of the event's block.
    #[schema(example = 2000000)]
    pub spec_version: u32,
    /// Status of the event's block.
    pub block_status: BlockStatus,
    /// Index of the event's trace, if traces are enabled.
    #[schema(required = false, nullable = false, example = 78)]
    pub trace_index: Option<u32>,
    /// Pallet index of the event.
    #[schema(example = 14)]
    pub pallet_index: u32,
    /// Pallet name of the event.
    #[schema(example = "Staking")]
    pub pallet_name: String,
    /// Index of the event in its pallet.
    #[schema(example = 12)]
    pub pallet_event_index: u32,
    /// Name of the event.
    #[schema(example = "Rewarded")]
    pub pallet_event_name: String,
    /// Index of the event's extrinsic, if it was output from an extrinsic.
    #[schema(required = false, nullable = false, example = 3)]
    pub extrinsic_index: Option<u32>,
    /// Hash of the event's extrinsic, if it was output from an extrinsic.
    #[schema(required = false, nullable = false)]
    pub extrinsic_hash: Option<Hash256Hex>,
    /// Event phase.
    #[schema(example = "ApplyExtrinsic")]
    pub phase: String,
    /// Index of the event in block.
    #[schema(example = 77)]
    pub index: u32,
    /// Event arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(schema_with = event_args_schema)]
    pub args: Option<JSONValue>,
}

impl From<&EventCompositeRow> for EventDTO {
    fn from(row: &EventCompositeRow) -> Self {
        Self {
            hash: Hash256Hex(format!("0x{}", hex::encode(&row.hash))),
            block_hash: Hash256Hex(format!("0x{}", hex::encode(&row.block_hash))),
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
                .map(|hash| Hash256Hex(format!("0x{}", hex::encode(hash)))),
            phase: row.phase.clone(),
            index: row.index as u32,
            args: row.args.clone(),
        }
    }
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of matching events.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct PaginatedEventList {
    #[schema(example = json!([event_example()]))]
    pub data: Vec<EventDTO>,
    pub pagination: PaginationData,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EventCursorPosition {
    pub(crate) block_number: u64,
    pub(crate) block_hash_hex: String,
    pub(crate) index: u32,
}

impl EventCursorPosition {
    pub(crate) fn get_block_hash(&self) -> anyhow::Result<Vec<u8>> {
        Ok(hex::decode(self.block_hash_hex.trim_start_matches("0x"))?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EventCursorPayload {
    pub(crate) cursor_position: EventCursorPosition,
    pub(crate) query: EventQuery,
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "List of matching events, with a cursor for the next page.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct CursorEventList {
    #[schema(example = json!([event_example()]))]
    pub data: Vec<EventDTO>,
    pub pagination: CursorPaginationData,
}

/// Event arguments wrapper.
#[derive(Debug, Serialize, ToSchema)]
#[schema(value_type = Object)]
pub(crate) struct EventArgs(pub JSONValue);

#[derive(Debug, Serialize, ToResponse)]
#[response(
    description = "List of matching events.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct EventList(pub Vec<EventDTO>);

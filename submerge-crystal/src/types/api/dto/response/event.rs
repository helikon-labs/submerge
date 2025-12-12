use serde::Serialize;
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::{
    api::dto::{pagination::PaginationData, response::hex::Hash256Hex},
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
pub struct EventDTO {
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
pub struct PaginatedEventList {
    #[schema(example = json!([event_example()]))]
    pub data: Vec<EventDTO>,
    pub pagination: PaginationData,
}

/// Event arguments wrapper.
#[derive(Debug, Serialize, ToSchema)]
#[schema(
    example = json!({
        "hash": "0x2c923bb54d06dfb649aaaf1c198eb1af9e19ec52b8e90267984496c128ee7adc",
        "args": {
            "to": "0x967cccc1ff3d1f37b9e6c8a39d8ba72ad85d35e19cc0717a72f1a21037606144",
            "from": "0x96b4be4ad947987922c88449866e738b4f4d09dece5157d2c3ac9477d8c6512e",
            "amount": "171162271"
        }
    }),
)]
pub struct EventArgs {
    /// Event hash.
    pub hash: Hash256Hex,
    #[schema(schema_with = event_args_schema)]
    pub args: JSONValue,
}

#[derive(Debug, Serialize, ToResponse)]
#[response(
    description = "List of matching events.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct EventList(pub Vec<EventDTO>);

fn event_args_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "to": "0x967cccc1ff3d1f37b9e6c8a39d8ba72ad85d35e19cc0717a72f1a21037606144",
            "from": "0x96b4be4ad947987922c88449866e738b4f4d09dece5157d2c3ac9477d8c6512e",
            "amount": "171162271"
        }))])
        .description(Some(
            "Event arguments in JSON format. Schema depends on runtime metadata and the event."
                .to_string(),
        ))
        .build()
}

fn event_example() -> JSONValue {
    let event = EventDTO {
        hash: Hash256Hex(
            "0x2c923bb54d06dfb649aaaf1c198eb1af9e19ec52b8e90267984496c128ee7adc".to_string(),
        ),
        block_hash: Hash256Hex(
            "0x5c4de7f2cea658d5d3804d495e8246354f709735d371fd54caaf59e80181bcaa".to_string(),
        ),
        block_number: 10758052,
        block_timestamp: Some(1765456362000),
        spec_version: 2000003,
        block_status: BlockStatus::Proposed,
        trace_index: Some(78),
        pallet_index: 0,
        pallet_name: "System".to_string(),
        pallet_event_index: 0,
        pallet_event_name: "ExtrinsicSuccess".to_string(),
        extrinsic_index: Some(0),
        extrinsic_hash: Some(Hash256Hex(
            "0x6963ce866a54258d9d6ca9222060f7270a8f5f6b83eaac88e899bb73fbbb68cb".to_string(),
        )),
        phase: "ApplyExtrinsic".to_string(),
        index: 1,
    };
    serde_json::to_value(&event).unwrap()
}

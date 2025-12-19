use serde_json::Value as JSONValue;

use crate::types::{
    api::dto::response::{event::EventDTO, hex::Hash256Hex},
    BlockStatus,
};

pub fn event_example() -> JSONValue {
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
        args: Some(serde_json::json!({
            "dispatchInfo": {
                "class": {
                    "type": "Mandatory",
                    "value": []
                },
                "paysFee": {
                    "type": "Yes",
                    "value": []
                },
                "weight": {
                    "proofSize": "0",
                    "refTime": "125000000"
                }
            }
        })),
    };
    serde_json::to_value(&event).unwrap()
}

use serde_json::Value as JSONValue;
use submerge_base::types::substrate::trace::TraceStorageMethod;

use crate::types::api::dto::response::{
    hex::{Hash256Hex, HexString},
    trace::TraceDTO,
};

pub(crate) fn trace_example() -> JSONValue {
    let event = TraceDTO {
        hash: Hash256Hex("0xebae2efe1479e7e4535c8ffb337359e252a54ac77a13c6095c9bbc5e78622daa".to_string()),
        block_hash: Hash256Hex("0x2f08d6887f29369af351118631221891b47ca5f0c2ef14f4da0dd32c3bed0d77".to_string()),
        block_number: 3213251,
        spec_version: 2000000,
        index: 20,
        key_prefix: HexString("0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9".to_string()),
        key_params: Some(HexString("0x5a3fb8de4321e12fad081eaeece61bc56d6f646c506f745374616b650000000000000000000000000000000000000000".to_string())),
        ext_id: HexString("0x3e44".to_string()),
        storage_method: TraceStorageMethod::Put,
        parent_id: None,
        known_key: None,
        pallet_index: Some(0),
        pallet_name: Some("System".to_string()),
        pallet_storage_item_index: Some(0),
        pallet_storage_item_name: Some("Account".to_string()),
    };
    serde_json::to_value(&event).unwrap()
}

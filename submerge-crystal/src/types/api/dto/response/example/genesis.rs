use serde_json::Value as JSONValue;

use crate::types::api::dto::response::{genesis::GenesisRecordDTO, hex::HexString};

pub(crate) fn genesis_record_example() -> JSONValue {
    let event = GenesisRecordDTO {
        key_prefix: HexString(
            "0x26aa394eea5630e07c48ae0c9558cef7a44704b568d21667356a5a050c118746".to_string(),
        ),
        key_params: Some(HexString("0xb4def25cfda6ef3a00000000".to_string())),
        value: HexString(
            "0x4545454545454545454545454545454545454545454545454545454545454545".to_string(),
        ),
        known_key: None,
        pallet_index: Some(0),
        pallet_name: Some("System".to_string()),
        pallet_storage_item_index: Some(4),
        pallet_storage_item_name: Some("BlockHash".to_string()),
    };
    serde_json::to_value(&event).unwrap()
}

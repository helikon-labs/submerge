use serde::Serialize;
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::api::dto::{pagination::PaginationData, response::hex::HexString};

/// A storage item initialized at genesis.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = GenesisRecord,
    example = genesis_record_example,
)]
pub struct GenesisRecordDTO {
    /// Storage item key prefix for the genesis record.
    pub key_prefix: HexString,
    /// Storage item key parameter(s) for the genesis record.
    #[schema(required = false, nullable = false)]
    pub key_params: Option<HexString>,
    /// Value of the genesis record.
    pub value: HexString,
    /// If the record is a known UTF-8 key, the string representation of the key.
    #[schema(required = false, nullable = false, example = ":extrinsic_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_key: Option<String>,
    /// If the record is a storage item, the storage item's pallet index in the metadata.
    #[schema(required = false, nullable = false, example = 0)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_index: Option<u32>,
    /// If the record is a storage item, the storage item's pallet name in the metadata.
    #[schema(required = false, nullable = false, example = "System")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_name: Option<String>,
    /// If the record is a storage item, the storage item's index in the pallet metadata.
    #[schema(required = false, nullable = false, example = 4)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_storage_item_index: Option<u32>,
    /// If the record is a storage item, the storage item's name.
    #[schema(required = false, nullable = false, example = "BlockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_storage_item_name: Option<String>,
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of genesis record items.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct PaginatedGenesisRecordList {
    #[schema(example = json!([genesis_record_example()]))]
    pub data: Vec<GenesisRecordDTO>,
    pub pagination: PaginationData,
}

fn genesis_record_example() -> JSONValue {
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

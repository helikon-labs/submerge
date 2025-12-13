use serde_json::Value as JSONValue;

use crate::types::api::dto::response::{
    hex::HexString,
    metadata::{
        MetadataCallDTO, MetadataConstantDTO, MetadataErrorDTO, MetadataEventDTO,
        MetadataItemDocumentation, MetadataPalletDTO, MetadataStorageItemDTO, MetadataSummaryDTO,
    },
};

pub fn metadata_summary_example() -> JSONValue {
    let metadata = MetadataSummaryDTO {
        spec_version: 1001,
        metadata_version: 14,
    };
    serde_json::to_value(&metadata).unwrap()
}

pub fn metadata_pallet_example() -> JSONValue {
    let pallet = MetadataPalletDTO {
        index: 0,
        name: "System".to_string(),
        calls: [MetadataCallDTO {
            index: 0,
            name: "Remark".to_string(),
            docs: MetadataItemDocumentation(["Make some on-chain remark.".to_string()].to_vec()),
        }]
        .to_vec(),
        constants: [MetadataConstantDTO {
            index: 1,
            name: "BlockLength".to_string(),
            type_id: Some(156),
            type_name: "frame_system::limits::BlockLength".to_string(),
            value_hex: HexString("0x00003c000000500000005000".to_string()),
            value: Some(serde_json::json!({
                "max": {
                    "mandatory": "5242880",
                    "normal": "3932160",
                    "operational": "5242880"
                }
            })),
            docs: MetadataItemDocumentation(
                ["The maximum length of a block (in bytes).".to_string()].to_vec(),
            ),
        }]
        .to_vec(),
        errors: [MetadataErrorDTO {
            index: 0,
            name: "InvalidSpecName".to_string(),
            docs: MetadataItemDocumentation(
                [
                    "The name of specification does not match between the current runtime"
                        .to_string(),
                    "and the new runtime.".to_string(),
                ]
                .to_vec(),
            ),
        }]
        .to_vec(),
        events: [MetadataEventDTO {
            index: 2,
            name: "CodeUpdated".to_string(),
            docs: MetadataItemDocumentation(["`:code` was updated.".to_string()].to_vec()),
        }]
        .to_vec(),
        storage_items: [MetadataStorageItemDTO {
            index: 8,
            name: "ParentHash".to_string(),
            key_prefix: HexString(
                "0x26aa394eea5630e07c48ae0c9558cef78a42f33323cb5ced3b44dd825fda9fcc".to_string(),
            ),
            docs: MetadataItemDocumentation(["Hash of the previous block.".to_string()].to_vec()),
        }]
        .to_vec(),
    };
    serde_json::to_value(&pallet).unwrap()
}

use serde_json::Value as JSONValue;

use crate::types::{
    api::dto::response::{call::CallDTO, hex::Hash256Hex},
    BlockStatus,
};

pub fn call_example() -> JSONValue {
    let call = CallDTO {
        hash: Hash256Hex(
            "0xf54535692c6b98bf9198d98ee28af8bc4d0753da6d42912042561dd8d32f4eca".to_string(),
        ),
        block_hash: Hash256Hex(
            "0x758fadeb5004882de8ba39ee2105302ad0ce93ecd68fe26b6fa09de6608e7a77".to_string(),
        ),
        block_number: 3172595,
        block_timestamp: Some(1765432302000),
        spec_version: 2000000,
        block_status: BlockStatus::Proposed,
        extrinsic_index: 0,
        extrinsic_hash: Hash256Hex(
            "0x18acc73c8e38351bc5b266cffacf39944945dd66342dab8ce2f86f2c97b3006f".to_string(),
        ),
        parent_call_hash: Some(Hash256Hex(
            "0x9bacc73c8e38351bc5b756cffacf39944945dd66342dab8ce2f86f2c97b3006f".to_string(),
        )),
        call_path: "root".to_string(),
        call_index: vec![0],
        pallet_index: 1,
        pallet_name: "Balances".to_string(),
        pallet_call_index: 0,
        pallet_call_name: "TransferKeepAlive".to_string(),
        extrinsic_is_successful: true,
    };
    serde_json::to_value(&call).unwrap()
}

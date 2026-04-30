use serde_json::Value as JSONValue;

use crate::types::{
    api::dto::response::{call::CallDTO, hex::Hash256Hex},
    BlockStatus,
};

pub(crate) fn call_example() -> JSONValue {
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
        call_name: "TransferKeepAlive".to_string(),
        extrinsic_is_successful: true,
        extrinsic_is_signed: true,
        is_successful: true,
        args: Some(serde_json::json!({
            "hash": "0xb778a81c1fd06d98b5ba1b37bb274101f7905ad5eca960f56ededf26248c4011",
            "args": {
                "dest": {
                    "type": "Id",
                    "value": "0xc35b9a45aadc8bb998ba7c4d17bda4d7d8e31f90a754a65709d3a3a71ff8fa7a"
                },
                "value": "117284000000"
            }
        })),
    };
    serde_json::to_value(&call).unwrap()
}

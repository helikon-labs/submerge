use serde_json::Value as JSONValue;

use crate::types::{
    api::dto::response::{
        block::BlockDTO,
        hex::{AccountIdHex, Hash256Hex},
        multi_address::{MultiAddressAccountIdDTO, MultiAddressAccountIdType, MultiAddressDTO},
    },
    BlockStatus,
};

pub fn block_example() -> JSONValue {
    let block = BlockDTO {
        hash: Hash256Hex(
            "0xc82fe0d5752d42ae3d325f14206859f86cec7447f244d5b4bccfc2a00bd58df8".to_string(),
        ),
        parent_hash: Hash256Hex(
            "0x1615581259dd1ac45fea1b23406367ca79c9f6dfa3b3b1115517c6e86250c42b".to_string(),
        ),
        state_root: Hash256Hex(
            "0x8c8b0b599733c41bad79a617d8f2f0213a5d965d287cee16c9efd65f23001603".to_string(),
        ),
        extrinsic_root: Hash256Hex(
            "0x7893dd573a5033a6d785bf4038c237cbd8e1f3730d177d4f9b21c8d2c7b34454".to_string(),
        ),
        number: 27419831,
        timestamp: Some(1755773684012),
        spec_version: 1006001,
        status: BlockStatus::Finalized,
        weight: Some(serde_json::json!({
            "normal": {
                "refTime": "0",
                "proofSize": "0"
            },
            "mandatory": {
                "refTime": "701777135384",
                "proofSize": "204955"
            },
            "operational": {
                "refTime": "0",
                "proofSize": "0"
            }
        })),
        extrinsic_count: 2,
        event_count: 56,
        author: Some(MultiAddressDTO::AccountId(MultiAddressAccountIdDTO {
            r#type: MultiAddressAccountIdType::AccountId,
            value: AccountIdHex(
                "0x269a84431cd8dfc5762beadfa54a8f21597c12d4f31e51f9f6f985f65ba0c626".to_string(),
            ),
        })),
    };
    serde_json::to_value(&block).unwrap()
}

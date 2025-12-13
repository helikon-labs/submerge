use serde_json::Value as JSONValue;

use crate::types::{
    api::dto::response::{
        extrinsic::ExtrinsicDTO,
        hex::{AccountIdHex, Hash256Hex, SignatureHexString},
        multi_address::{MultiAddressAccountIdDTO, MultiAddressAccountIdType, MultiAddressDTO},
        multi_signature::{MultiSignatureDTO, MultiSignatureSr25519DTO, MultiSignatureSr25519Type},
    },
    BlockStatus,
};

pub fn extrinsic_example() -> JSONValue {
    let extrinsic = ExtrinsicDTO {
        block_hash: Hash256Hex(
            "0x758fadeb5004882de8ba39ee2105302ad0ce93ecd68fe26b6fa09de6608e7a77".to_string(),
        ),
        block_number: 3172595,
        block_timestamp: Some(1765432302000),
        spec_version: 2000000,
        block_status: BlockStatus::Proposed,
        trace_index: Some(2),
        hash: Hash256Hex("0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3".to_string()),
        index: 2,
        version: 4,
        signer: Some(MultiAddressDTO::AccountId(MultiAddressAccountIdDTO {
            r#type: MultiAddressAccountIdType::AccountId,
            value: AccountIdHex(
                "0x269a84431cd8dfc5762beadfa54a8f21597c12d4f31e51f9f6f985f65ba0c626".to_string(),
            ),
        })),
        signature: Some(MultiSignatureDTO::Sr25519(MultiSignatureSr25519DTO {
            r#type: MultiSignatureSr25519Type::Sr25519,
            value: SignatureHexString(
                "0xabababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababab".to_string()
            ),
        })),
        is_successful: true,
        extra: Some(serde_json::json!({
            "checkNonce": "8362",
            "checkWeight": {},
            "checkGenesis": {},
            "checkMortality": {
                "type": "Mortal84",
                "value": "0"
            },
            "checkTxVersion": {},
            "checkSpecVersion": {},
            "checkMetadataHash": {
                "mode": {
                    "type": "Disabled",
                    "value": []
                }
            },
            "checkNonZeroSender": {},
            "chargeAssetTxPayment": {
                "tip": "0",
                "assetId": null
            }
        })),
    };
    serde_json::to_value(&extrinsic).unwrap()
}

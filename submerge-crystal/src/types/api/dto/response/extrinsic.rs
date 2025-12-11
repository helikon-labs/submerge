use parity_scale_codec::Decode as _;
use serde::Serialize;
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::{
    api::dto::{
        pagination::PaginationData,
        response::{
            hex::{AccountIdHex, Hash256Hex, SignatureHexString},
            multi_address::{MultiAddressAccountIdDTO, MultiAddressAccountIdType, MultiAddressDTO},
            multi_signature::{
                MultiSignatureDTO, MultiSignatureSr25519DTO, MultiSignatureSr25519Type,
            },
        },
    },
    persistence::ExtrinsicRow,
    BlockStatus,
};
use submerge_base::types::substrate::{
    multi_address::MultiAddress, multi_signature::MultiSignature,
};

/// An extrinsic in a block. Signed or unsigned.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = Extrinsic,
    example = extrinsic_example,
)]
pub struct ExtrinsicDTO {
    /// Hash of the extrinsic's block.
    pub block_hash: Hash256Hex,
    /// Number of the extrinsic's block.
    pub block_number: u64,
    /// Timestamp of the extrinsic's block. Milliseconds.
    #[schema(required = false, nullable = false)]
    pub block_timestamp: Option<u64>,
    /// Runtime spec version of the extrinsic's block.
    pub spec_version: u32,
    /// Status of the extrinsic's block.
    pub block_status: BlockStatus,
    /// Trace index for the extrinsic.
    #[schema(required = false, nullable = false)]
    pub trace_index: Option<u32>,
    /// Extrinsic hash.
    pub hash: Hash256Hex,
    /// Extrinsic index.
    pub index: u32,
    /// Extrinsic version in metadata.
    pub version: u32,
    /// Extrinsic signer address.
    #[schema(required = false, nullable = false)]
    pub signer: Option<MultiAddressDTO>,
    /// Extrinsic signature address.
    #[schema(required = false, nullable = false)]
    pub signature: Option<MultiSignatureDTO>,
    /// Whether the extrinsic was successful.
    pub is_successful: bool,
    #[schema(
        required = false,
        nullable = false,
        schema_with = extrinsic_extra_schema,
    )]
    pub extra: Option<JSONValue>,
}

impl TryFrom<&ExtrinsicRow> for ExtrinsicDTO {
    type Error = anyhow::Error;

    fn try_from(row: &ExtrinsicRow) -> Result<Self, Self::Error> {
        let signer_multi_address = if let Some(bytes) = row.signer_multi_address.as_ref() {
            let mut bytes: &[u8] = &bytes.clone();
            let multi_address = MultiAddress::decode(&mut bytes)?;
            Some(multi_address)
        } else {
            None
        };
        let multi_signature = if let Some(bytes) = row.multi_signature.as_ref() {
            let mut bytes: &[u8] = &bytes.clone();
            let multi_signature = MultiSignature::decode(&mut bytes)?;
            Some(multi_signature)
        } else {
            None
        };
        Ok(Self {
            block_hash: Hash256Hex(format!("0x{}", hex::encode(&row.block_hash))),
            block_number: row.block_number as u64,
            block_timestamp: row.block_timestamp.map(|timestamp| timestamp as u64),
            spec_version: row.spec_version as u32,
            block_status: row.block_status,
            trace_index: row.trace_index.map(|i| i as u32),
            hash: Hash256Hex(format!("0x{}", hex::encode(row.hash))),
            index: row.index as u32,
            version: row.version as u32,
            signer: signer_multi_address
                .as_ref()
                .map(|multi_address| multi_address.into()),
            signature: multi_signature
                .as_ref()
                .map(|multi_signature| multi_signature.into()),
            is_successful: row.is_successful,
            extra: row.extra.clone(),
        })
    }
}

#[derive(Debug, Serialize, ToResponse)]
#[response(
    description = "List of matching extrinsics.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct ExtrinsicList(pub Vec<ExtrinsicDTO>);

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of matching extrinsics.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct PaginatedExtrinsicList {
    #[schema(example = json!([extrinsic_example()]))]
    pub data: Vec<ExtrinsicDTO>,
    pub pagination: PaginationData,
}

fn extrinsic_extra_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
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
          }))])
        .description(Some(
            "Extrinsic extras in JSON format - checkNonce, checkGenesis, chargeTransactionPayment, etc.".to_string(),
        ))
        .build()
}

fn extrinsic_example() -> JSONValue {
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

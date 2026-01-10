use parity_scale_codec::Decode as _;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::{
    api::dto::{
        pagination::{CursorPaginationData, PaginationData},
        request::extrinsic::ExtrinsicQuery,
        response::{
            example::extrinsic::extrinsic_example, hex::Hash256Hex, multi_address::MultiAddressDTO,
            multi_signature::MultiSignatureDTO, schema::extrinsic::extrinsic_extra_schema,
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
pub(crate) struct ExtrinsicDTO {
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
pub(crate) struct ExtrinsicList(pub Vec<ExtrinsicDTO>);

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of matching extrinsics.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct PaginatedExtrinsicList {
    #[schema(example = json!([extrinsic_example()]))]
    pub data: Vec<ExtrinsicDTO>,
    pub pagination: PaginationData,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExtrinsicCursorPosition {
    pub(crate) block_number: u64,
    pub(crate) block_hash_hex: String,
    pub(crate) index: u32,
}

impl ExtrinsicCursorPosition {
    pub(crate) fn get_block_hash(&self) -> anyhow::Result<Vec<u8>> {
        Ok(hex::decode(self.block_hash_hex.trim_start_matches("0x"))?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExtrinsicCursorPayload {
    pub(crate) cursor_position: ExtrinsicCursorPosition,
    pub(crate) query: ExtrinsicQuery,
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "List of matching extrinsics, with a cursor for the next page.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct CursorExtrinsicList {
    #[schema(example = json!([extrinsic_example()]))]
    pub data: Vec<ExtrinsicDTO>,
    pub pagination: CursorPaginationData,
}

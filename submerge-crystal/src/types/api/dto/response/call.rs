use serde::Serialize;
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::{
    api::dto::{pagination::PaginationData, response::hex::Hash256Hex},
    persistence::CallRow,
    BlockStatus,
};

/// A runtime call.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = Call,
    example = call_example,
)]
pub struct CallDTO {
    /// Artificial call hash.
    pub hash: Hash256Hex,
    /// Hash of the call's block.
    pub block_hash: Hash256Hex,
    /// Number of the call's block.
    #[schema(example = 3172595)]
    pub block_number: u64,
    /// Timestamp of the call's block. Milliseconds.
    #[schema(required = false, nullable = false, example = 1755773684012u64)]
    pub block_timestamp: Option<u64>,
    /// Runtime spec version of the call's block.
    #[schema(example = 2000000)]
    pub spec_version: u32,
    /// Status of the call's block.
    pub block_status: BlockStatus,
    /// Index of the call's extrinsic.
    #[schema(example = 1)]
    pub extrinsic_index: u32,
    /// Hash of the call's extrinsic.
    pub extrinsic_hash: Hash256Hex,
    /// Hash of the parent call if this call is a nested call (batch, multisig, etc.).
    #[schema(required = false, nullable = false)]
    pub parent_call_hash: Option<Hash256Hex>,
    /// Rust-style path of the call represented with parameter names or indices.
    #[schema(example = "root::calls:0")]
    pub call_path: String,
    /// Index of the call, represented as an unsigned integer array.
    #[schema(example = json!([0, 1, 0]))]
    pub call_index: Vec<u16>,
    /// Pallet index of the call.
    #[schema(example = 14)]
    pub pallet_index: u32,
    /// Pallet name of the call.
    #[schema(example = "System")]
    pub pallet_name: String,
    /// Index of the call in its pallet.
    #[schema(example = 5)]
    pub pallet_call_index: u32,
    /// Name of the call.
    #[schema(example = "SetCode")]
    pub pallet_call_name: String,
    /// Whether the call's extrinsic was successful.
    /// Note: The extrinsic can be successful where the call has failed (see the Utility.ForceBatch call).
    pub extrinsic_is_successful: bool,
}

impl From<&CallRow> for CallDTO {
    fn from(row: &CallRow) -> Self {
        Self {
            hash: Hash256Hex(format!("0x{}", hex::encode(&row.hash))),
            block_hash: Hash256Hex(format!("0x{}", hex::encode(&row.block_hash))),
            block_number: row.block_number as u64,
            block_timestamp: row.block_timestamp.map(|timestamp| timestamp as u64),
            spec_version: row.spec_version as u32,
            block_status: row.block_status,
            extrinsic_index: row.extrinsic_index as u32,
            extrinsic_hash: Hash256Hex(format!("0x{}", hex::encode(row.extrinsic_hash))),
            parent_call_hash: row
                .parent_call_hash
                .as_deref()
                .map(|parent_call_hash| Hash256Hex(format!("0x{}", hex::encode(parent_call_hash)))),
            call_path: row.call_path.clone(),
            call_index: row
                .call_index
                .iter()
                .map(|&x| x as u16) // wrapping conversion
                .collect(),
            pallet_index: row.pallet_index as u32,
            pallet_name: row.pallet_name.clone(),
            pallet_call_index: row.pallet_call_index as u32,
            pallet_call_name: row.pallet_call_name.clone(),
            extrinsic_is_successful: row.extrinsic_is_successful,
        }
    }
}

#[derive(Debug, Serialize, ToResponse)]
#[response(
    description = "List of matching calls.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct CallList(pub Vec<CallDTO>);

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of matching calls.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct PaginatedCallList {
    #[schema(example = json!([call_example()]))]
    pub data: Vec<CallDTO>,
    pub pagination: PaginationData,
}

fn call_example() -> JSONValue {
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
        pallet_name: "ParachainSystem".to_string(),
        pallet_call_index: 0,
        pallet_call_name: "SetValidationData".to_string(),
        extrinsic_is_successful: true,
    };
    serde_json::to_value(&call).unwrap()
}

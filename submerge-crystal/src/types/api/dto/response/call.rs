use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::{
    api::dto::{
        pagination::{CursorPaginationData, PaginationData},
        request::call::CallQuery,
        response::{example::call::call_example, hex::Hash256Hex, schema::call::call_args_schema},
    },
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
pub(crate) struct CallDTO {
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
    pub call_name: String,
    /// Whether the call's extrinsic was successful.
    /// Note: The extrinsic can be successful where the call has failed (see the `Utility.ForceBatch`` call).
    pub extrinsic_is_successful: bool,
    /// Whether the call's extrinsic was signed.
    pub extrinsic_is_signed: bool,
    /// Whether the call was successful.
    pub is_successful: bool,
    /// Call arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(schema_with = call_args_schema)]
    pub args: Option<JSONValue>,
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
            call_name: row.call_name.clone(),
            extrinsic_is_successful: row.extrinsic_is_successful,
            extrinsic_is_signed: row.extrinsic_is_signed,
            is_successful: row.is_successful,
            args: row.args.clone(),
        }
    }
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of matching calls.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct PaginatedCallList {
    #[schema(example = json!([call_example()]))]
    pub data: Vec<CallDTO>,
    pub pagination: PaginationData,
}

/// Call arguments wrapper.
#[derive(Debug, Serialize, ToSchema)]
#[schema(value_type = Object)]
pub(crate) struct CallArgs(pub JSONValue);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CallCursorPosition {
    pub(crate) block_number: u64,
    pub(crate) block_hash_hex: String,
    pub(crate) call_index: Vec<u16>,
}

impl CallCursorPosition {
    pub(crate) fn get_block_hash(&self) -> anyhow::Result<Vec<u8>> {
        Ok(hex::decode(self.block_hash_hex.trim_start_matches("0x"))?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CallCursorPayload {
    pub(crate) cursor_position: CallCursorPosition,
    pub(crate) query: CallQuery,
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "List of matching events, with a cursor for the next page.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct CursorCallList {
    #[schema(example = json!([call_example()]))]
    pub data: Vec<CallDTO>,
    pub pagination: CursorPaginationData,
}

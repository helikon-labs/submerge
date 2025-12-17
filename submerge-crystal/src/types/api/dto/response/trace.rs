use serde::Serialize;
use std::string;
use submerge_base::types::substrate::trace::TraceStorageMethod;
use utoipa::{ToResponse, ToSchema};

use crate::types::{
    api::dto::{
        pagination::PaginationData,
        response::{
            example::trace::trace_example,
            hex::{Hash256Hex, HexString},
        },
    },
    persistence::TraceRow,
};

/// A trace record in a block.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = Trace,
    example = trace_example,
)]
pub struct TraceDTO {
    /// Artificial hash of the trace (`sha256(block_hash || index)`).
    pub hash: Hash256Hex,
    /// Hash of the trace's block.
    pub block_hash: Hash256Hex,
    /// Number of the trace's block.
    #[schema(example = 3172595)]
    pub block_number: u64,
    /// Runtime spec version of the call's block.
    #[schema(example = 2000000)]
    pub spec_version: u32,
    /// Index of the trace within the block.
    #[schema(example = 83)]
    pub index: u32,
    /// Substrate storage key prefix of the trace record.
    pub key_prefix: HexString,
    /// Substrate storage parameters appended to the end of the storage key,
    /// if the storage item has parameters.
    #[schema(required = false, nullable = false)]
    pub key_params: Option<HexString>,
    /// ExtId value for the trace record.
    pub ext_id: HexString,
    /// Trace storage method.
    pub storage_method: TraceStorageMethod,
    pub parent_id: Option<String>,
    /// If the trace record is for a known UTF-8 key, the string representation of the key.
    #[schema(required = false, nullable = false, example = ":extrinsic_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_key: Option<String>,
    /// If the trace record is a storage item, the storage item's pallet index in the metadata.
    #[schema(required = false, nullable = false, example = 14)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_index: Option<u32>,
    /// If the trace record is a storage item, the storage item's pallet name in the metadata.
    #[schema(required = false, nullable = false, example = "System")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_name: Option<String>,
    /// If the trace record is a storage item, the storage item's index in the pallet metadata.
    #[schema(required = false, nullable = false, example = 4)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_storage_item_index: Option<u32>,
    /// If the trace record is a storage item, the storage item's name.
    #[schema(required = false, nullable = false, example = "BlockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_storage_item_name: Option<String>,
}

impl TryFrom<&TraceRow> for TraceDTO {
    type Error = string::FromUtf8Error;

    fn try_from(row: &TraceRow) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: Hash256Hex(format!("0x{}", hex::encode(&row.hash))),
            block_hash: Hash256Hex(format!("0x{}", hex::encode(&row.block_hash))),
            block_number: row.block_number as u64,
            spec_version: row.spec_version as u32,
            index: row.index as u32,
            key_prefix: HexString(format!("0x{}", hex::encode(&row.key_prefix))),
            key_params: row
                .key_params
                .as_ref()
                .map(|key_params| HexString(format!("0x{}", hex::encode(key_params)))),
            ext_id: HexString(format!("0x{}", hex::encode(&row.ext_id))),
            storage_method: row.storage_method.clone(),
            parent_id: row.parent_id.clone(),
            known_key: if row.is_known_key {
                Some(String::from_utf8(row.key_prefix.clone())?)
            } else {
                None
            },
            pallet_index: row.pallet_index.map(|index| index as u32),
            pallet_name: row.pallet_name.clone(),
            pallet_storage_item_index: row.pallet_storage_item_index.map(|index| index as u32),
            pallet_storage_item_name: row.pallet_storage_item_name.clone(),
        })
    }
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of matching traces.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct PaginatedTraceList {
    #[schema(example = json!([trace_example()]))]
    pub data: Vec<TraceDTO>,
    pub pagination: PaginationData,
}

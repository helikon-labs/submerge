use serde::Serialize;
use utoipa::ToResponse;

use crate::types::api::dto::{
    pagination::PaginationData,
    response::{example::block::block_example, schema::block::block_weight_schema},
};

use parity_scale_codec::Decode as _;
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::multi_address::MultiAddress;
use utoipa::ToSchema;

use super::{hex::Hash256Hex, multi_address::MultiAddressDTO};

use crate::types::{persistence::BlockRow, BlockStatus};

/// A block in the blockchain.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = Block,
    example = block_example,
)]
pub struct BlockDTO {
    /// Block hash (Blake2 256-bit).
    pub hash: Hash256Hex,
    /// Parent block hash.
    pub parent_hash: Hash256Hex,
    /// Root hash of the state trie after executing this block.
    pub state_root: Hash256Hex,
    /// Merkle root of included extrinsics.
    pub extrinsic_root: Hash256Hex,
    /// Block height.
    #[schema(example = 27419831u64)]
    pub number: u64,
    /// Block timestamp. Milliseconds.
    #[schema(required = false, nullable = false, example = 1755773684012u64)]
    pub timestamp: Option<u64>,
    /// Runtime spec version.
    #[schema(example = 1006001)]
    pub spec_version: u32,
    /// Block status.
    #[schema(example = "finalized")]
    pub status: BlockStatus,
    #[schema(
        required = false,
        nullable = false,
        schema_with = block_weight_schema,
    )]
    pub weight: Option<JSONValue>,
    /// Number of extrinsics in the block.
    #[schema(minimum = 0, example = 2)]
    pub extrinsic_count: u32,
    /// Number of events in the block.
    #[schema(minimum = 0, example = 56)]
    pub event_count: u32,
    /// Authoring validator address.
    #[schema(required = false, nullable = false)]
    pub author: Option<MultiAddressDTO>,
}

impl TryFrom<&BlockRow> for BlockDTO {
    type Error = anyhow::Error;

    fn try_from(row: &BlockRow) -> Result<Self, Self::Error> {
        let author_multi_address = if let Some(bytes) = row.author_multi_address.as_ref() {
            let mut bytes: &[u8] = &bytes.clone();
            let multi_address = MultiAddress::decode(&mut bytes)?;
            Some(multi_address)
        } else {
            None
        };
        Ok(Self {
            hash: row.hash.as_slice().into(),
            parent_hash: row.parent_hash.as_slice().into(),
            state_root: row.state_root.as_slice().into(),
            extrinsic_root: row.extrinsic_root.as_slice().into(),
            number: row.number as u64,
            timestamp: row.timestamp.map(|timestamp| timestamp as u64),
            spec_version: row.spec_version as u32,
            status: row.status,
            weight: row.weight.clone(),
            extrinsic_count: row.extrinsic_count as u32,
            event_count: row.event_count as u32,
            author: author_multi_address
                .as_ref()
                .map(|multi_address| multi_address.into()),
        })
    }
}

#[derive(Debug, Serialize, ToResponse)]
#[response(
    description = "List of matching blocks.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct BlockList(pub Vec<BlockDTO>);

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    description = "Paginated list of matching blocks.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct PaginatedBlockList {
    #[schema(example = json!([block_example()]))]
    pub data: Vec<BlockDTO>,
    pub pagination: PaginationData,
}

use std::str::FromStr as _;

use parity_scale_codec::Decode as _;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::multi_address::MultiAddress;
use utoipa::{IntoParams, ToSchema};

use crate::types::{
    api::dto::{
        block_weight_schema, multi_address::MultiAddressDTO, pagination::PaginationData, Hash256Hex,
    },
    persistence::BlockRow,
    BlockStatus,
};

pub enum BlockReference {
    Hash(Vec<u8>),
    Number(u64),
}

impl TryFrom<&str> for BlockReference {
    type Error = String;

    fn try_from(reference: &str) -> Result<Self, Self::Error> {
        if let Ok(number) = reference.parse::<u64>() {
            Ok(Self::Number(number))
        } else if let Ok(hash) = hex::decode(reference.trim_start_matches("0x")) {
            Ok(Self::Hash(hash))
        } else {
            Err("Invalid block reference. It should be either a block number (integer ≥ 0), or a block hash in hex (with or without 0x prefix, case-insensitive).".to_string())
        }
    }
}

/// Query parameters for fetching blocks.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct BlockQuery {
    /// Page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of items per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Filter results by block status. If not specified, all blocks are returned.
    #[param(required = false, nullable = false)]
    pub status: Option<BlockStatus>,
    /// Filter results by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    pub min_block_number: Option<u64>,
    /// Filter results by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    pub max_block_number: Option<u64>,
    /// Filter results by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub min_block_timestamp: Option<u64>,
    /// Filter results by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub max_block_timestamp: Option<u64>,
    /// Filter results by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub min_spec_version: Option<u32>,
    /// Filter results by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub max_spec_version: Option<u32>,
    /// Filter results by block author. Either of the following:
    /// - Author's Substrate SS58 address string (e.g. `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`).
    /// - Author's Substrate account id encoded as a hex string (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
    /// - Author's address encoded as a hex string with optional `0x` prefix (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
    #[param(
        required = false,
        max_length = 512,
        nullable = false,
        pattern = "^(?:[1-9A-HJ-NP-Za-km-z]{47,48}|(?:0x)?[0-9a-fA-F]{1-256})$",
        example = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    )]
    pub author: Option<String>,
}

impl BlockQuery {
    pub fn get_author_multi_address(&self) -> anyhow::Result<Option<MultiAddress>> {
        let author = if let Some(author) = &self.author {
            Some(MultiAddress::from_str(author)?)
        } else {
            None
        };
        Ok(author)
    }
}

fn block_dto_example() -> JSONValue {
    serde_json::json!({
        "hash": "0xc82fe0d5752d42ae3d325f14206859f86cec7447f244d5b4bccfc2a00bd58df8",
        "parentHash": "0x1615581259dd1ac45fea1b23406367ca79c9f6dfa3b3b1115517c6e86250c42b",
        "stateRoot": "0x8c8b0b599733c41bad79a617d8f2f0213a5d965d287cee16c9efd65f23001603",
        "extrinsicRoot": "0x7893dd573a5033a6d785bf4038c237cbd8e1f3730d177d4f9b21c8d2c7b34454",
        "number": 27419831,
        "timestamp": 1755773684012u64,
        "specVersion": 1006001,
        "status": "finalized",
        "weight": {
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
        },
        "extrinsicCount": 2,
        "eventCount": 56,
        "author": {
            "type": "accountId",
            "value": "0x269a84431cd8dfc5762beadfa54a8f21597c12d4f31e51f9f6f985f65ba0c626"
        }
    })
}

fn block_dto_list_example() -> JSONValue {
    serde_json::json!([block_dto_example()])
}

/// A block in the blockchain.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = Block,
    example = block_dto_example,
)]
pub struct BlockDTO {
    pub hash: Hash256Hex,
    pub parent_hash: Hash256Hex,
    pub state_root: Hash256Hex,
    pub extrinsic_root: Hash256Hex,
    /// Block height.
    #[schema(example = 27419831u64)]
    pub number: u64,
    /// Block timestamp - milliseconds.
    #[schema(required = false, nullable = false, example = 1755773684012u64)]
    pub timestamp: Option<u64>,
    /// Runtime spec version.
    #[schema(example = 1006001)]
    pub spec_version: u32,
    /// Block status.
    #[schema(example = "finalized")]
    pub status: BlockStatus,
    /// Block weight in JSON format. Schema depends on runtime metadata.
    #[schema(
        required = false,
        nullable = false,
        schema_with = block_weight_schema,
    )]
    pub weight: Option<String>,
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
            weight: if let Some(weight) = &row.weight {
                Some(serde_json::to_string(weight)?)
            } else {
                None
            },
            extrinsic_count: row.extrinsic_count as u32,
            event_count: row.event_count as u32,
            author: author_multi_address
                .as_ref()
                .map(|multi_address| multi_address.into()),
        })
    }
}

/// Paginated list of blocks in blockchain.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedBlockList {
    #[schema(
        example = block_dto_list_example,
        max_items = 100,
    )]
    pub data: Vec<BlockDTO>,
    pub pagination: PaginationData,
}

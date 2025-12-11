use serde::Serialize;
use utoipa::ToResponse;

use crate::types::api::dto::pagination::PaginationData;

use parity_scale_codec::Decode as _;
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::multi_address::MultiAddress;
use utoipa::ToSchema;

use super::{
    hex::{AccountIdHex, Hash256Hex},
    multi_address::{MultiAddressAccountIdDTO, MultiAddressAccountIdType, MultiAddressDTO},
};

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

fn block_weight_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "normal": {
                "refTime": "0",
                "proofSize": "0"
            },
            "mandatory": {
                "refTime": "361766342408",
                "proofSize": "592668"
            },
            "operational": {
                "refTime": "0",
                "proofSize": "0"
            },
        }))])
        .description(Some(
            "Block weight in JSON format. Schema depends on runtime metadata.".to_string(),
        ))
        .build()
}

fn block_example() -> JSONValue {
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

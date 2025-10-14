use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::account_id::AccountId;

use crate::types::{api::dto::pagination::PaginationQuery, persistence::BlockRow, BlockStatus};

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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub status: Option<BlockStatus>,
    pub min_block_number: Option<u64>,
    pub max_block_number: Option<u64>,
    pub min_block_timestamp: Option<u64>,
    pub max_block_timestamp: Option<u64>,
    pub min_spec_version: Option<u64>,
    pub max_spec_version: Option<u64>,
    pub author: Option<AccountId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockDTO {
    pub hash: String,
    pub parent_hash: String,
    pub state_root: String,
    pub extrinsic_root: String,
    pub number: u64,
    pub timestamp: Option<u64>,
    pub spec_version: u32,
    pub status: BlockStatus,
    pub weight: Option<JSONValue>,
    pub extrinsic_count: u32,
    pub event_count: u32,
    pub author_account_id: Option<String>,
}

impl From<&BlockRow> for BlockDTO {
    fn from(row: &BlockRow) -> Self {
        Self {
            hash: format!("0x{}", hex::encode(&row.hash)),
            parent_hash: format!("0x{}", hex::encode(&row.parent_hash)),
            state_root: format!("0x{}", hex::encode(&row.state_root)),
            extrinsic_root: format!("0x{}", hex::encode(&row.extrinsic_root)),
            number: row.number as u64,
            timestamp: row.timestamp.map(|timestamp| timestamp as u64),
            spec_version: row.spec_version as u32,
            status: row.status,
            weight: row.weight.clone(),
            extrinsic_count: row.extrinsic_count as u32,
            event_count: row.event_count as u32,
            author_account_id: row
                .author_account_id
                .as_ref()
                .map(|account_id| format!("0x{}", hex::encode(account_id))),
        }
    }
}

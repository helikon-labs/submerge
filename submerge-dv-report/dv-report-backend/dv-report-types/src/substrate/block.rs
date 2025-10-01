use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub timestamp: u64,
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
}

#[derive(Debug, FromRow)]
pub struct BlockRow {
    pub network_id: i32,
    pub hash: String,
    pub number: i64,
    pub timestamp: i64,
    pub parent_hash: String,
}

impl From<BlockRow> for Block {
    fn from(row: BlockRow) -> Self {
        Self {
            timestamp: row.timestamp as u64,
            number: row.number as u64,
            hash: row.hash.clone(),
            parent_hash: row.parent_hash.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventDigest {
    logs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub digest: EventDigest,
    pub extrinsics_root: String,
    pub number: String,
    pub parent_hash: String,
    pub state_root: String,
}

impl BlockHeader {
    pub fn get_number(&self) -> anyhow::Result<u64> {
        let number = u64::from_str_radix(self.number.trim_start_matches("0x"), 16)?;
        Ok(number)
    }
}

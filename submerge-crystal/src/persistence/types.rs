use crate::types::BlockStatus;
use sqlx::FromRow;

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct BlockRow {
    pub hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
    pub state_root: Vec<u8>,
    pub extrinsic_root: Vec<u8>,
    pub number: i64,
    pub timestamp: i64,
    pub spec_version: i32,
    pub status: BlockStatus,
    pub extrinsic_count: i32,
    pub event_count: i32,
    pub author_account_id: Vec<u8>,
}

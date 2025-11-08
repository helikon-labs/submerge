use parity_scale_codec::Encode as _;
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::{persistence::BlockRow, BlockStatus};

pub(crate) trait CrystalBlockAPIPostgreSQLStorage {
    async fn get_block_count(
        &self,
        status: Option<BlockStatus>,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        author_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64>;
    async fn get_block_rows(
        &self,
        status: Option<BlockStatus>,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        author_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<BlockRow>>;
}

impl CrystalBlockAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_block_count(
        &self,
        status: Option<BlockStatus>,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        author_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM block
            WHERE
                ($1 IS NULL or status = $1)
                AND ($2 IS NULL OR number >= $2)
                AND ($3 IS NULL OR number <= $3)
                AND ($4 IS NULL OR timestamp >= $4)
                AND ($5 IS NULL OR timestamp <= $5)
                AND ($6 IS NULL OR spec_version >= $6)
                AND ($7 IS NULL OR spec_version <= $7)
                AND ($8 IS NULL OR author_multi_address = $8)
            "#,
        )
        .bind(status)
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(min_block_timestamp.map(|n| n as i64))
        .bind(max_block_timestamp.map(|n| n as i64))
        .bind(min_spec_version.map(|n| n as i32))
        .bind(max_spec_version.map(|n| n as i32))
        .bind(
            author_multi_address
                .as_ref()
                .map(|multi_address| multi_address.encode()),
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_block_rows(
        &self,
        status: Option<BlockStatus>,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        author_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<BlockRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<BlockRow> = sqlx::query_as(
            r#"
            SELECT
                hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version,
                status, weight, extrinsic_count, event_count, author_multi_address
            FROM block
            WHERE
                ($1 IS NULL or status = $1)
                AND ($2 IS NULL OR number >= $2)
                AND ($3 IS NULL OR number <= $3)
                AND ($4 IS NULL OR timestamp >= $4)
                AND ($5 IS NULL OR timestamp <= $5)
                AND ($6 IS NULL OR spec_version >= $6)
                AND ($7 IS NULL OR spec_version <= $7)
                AND ($8 IS NULL OR author_multi_address = $8)
            ORDER BY number DESC
            LIMIT $9 OFFSET $10
            "#,
        )
        .bind(status)
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(min_block_timestamp.map(|n| n as i64))
        .bind(max_block_timestamp.map(|n| n as i64))
        .bind(min_spec_version.map(|n| n as i32))
        .bind(max_spec_version.map(|n| n as i32))
        .bind(
            author_multi_address
                .as_ref()
                .map(|multi_address| multi_address.encode()),
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }
}

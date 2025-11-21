use parity_scale_codec::Encode as _;
use sqlx::{Pool, Postgres};
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::{persistence::BlockRow, BlockStatus};

async fn get_max_number_before_timestamp(
    connection_pool: &Pool<Postgres>,
    timestamp: u64,
) -> anyhow::Result<Option<u64>> {
    let number: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT number
        FROM block
        WHERE timestamp <= $1
        ORDER BY timestamp DESC
        LIMIT 1
        "#,
    )
    .bind(timestamp as i64)
    .fetch_optional(connection_pool)
    .await?;
    Ok(number.map(|number| number as u64))
}

async fn get_min_number_after_timestamp(
    connection_pool: &Pool<Postgres>,
    timestamp: u64,
) -> anyhow::Result<Option<u64>> {
    let number: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT number
        FROM block
        WHERE timestamp >= $1
        ORDER BY timestamp ASC
        LIMIT 1
        "#,
    )
    .bind(timestamp as i64)
    .fetch_optional(connection_pool)
    .await?;
    Ok(number.map(|number| number as u64))
}

async fn get_min_number_with_spec_version(
    connection_pool: &Pool<Postgres>,
    spec_version: u32,
) -> anyhow::Result<Option<u64>> {
    let number: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT number
        FROM block
        WHERE spec_version = $1
        ORDER BY number ASC
        LIMIT 1
        "#,
    )
    .bind(spec_version as i32)
    .fetch_optional(connection_pool)
    .await?;
    Ok(number.map(|number| number as u64))
}

async fn get_max_number_with_spec_version(
    connection_pool: &Pool<Postgres>,
    spec_version: u32,
) -> anyhow::Result<Option<u64>> {
    let number: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT number
        FROM block
        WHERE spec_version = $1
        ORDER BY number DESC
        LIMIT 1
        "#,
    )
    .bind(spec_version as i32)
    .fetch_optional(connection_pool)
    .await?;
    Ok(number.map(|number| number as u64))
}

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
    async fn get_block_number_range(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
    ) -> anyhow::Result<(Option<u64>, Option<u64>)>;
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

    async fn get_block_number_range(
        &self,
        min_number: Option<u64>,
        max_number: Option<u64>,
        min_timestamp: Option<u64>,
        max_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
    ) -> anyhow::Result<(Option<u64>, Option<u64>)> {
        let min_timestamp_block_number = if let Some(min_timestamp) = min_timestamp {
            get_min_number_after_timestamp(&self.connection_pool, min_timestamp).await?
        } else {
            None
        };
        let max_timestamp_block_number = if let Some(max_timestamp) = max_timestamp {
            get_max_number_before_timestamp(&self.connection_pool, max_timestamp).await?
        } else {
            None
        };
        let min_spec_version_block_number = if let Some(min_spec_version) = min_spec_version {
            get_min_number_with_spec_version(&self.connection_pool, min_spec_version).await?
        } else {
            None
        };
        let max_spec_version_block_number = if let Some(max_spec_version) = max_spec_version {
            get_max_number_with_spec_version(&self.connection_pool, max_spec_version).await?
        } else {
            None
        };
        let min = min_number
            .unwrap_or(0)
            .max(min_timestamp_block_number.unwrap_or(0))
            .max(min_spec_version_block_number.unwrap_or(0));
        let max = max_number
            .unwrap_or(u64::MAX)
            .min(max_timestamp_block_number.unwrap_or(u64::MAX))
            .min(max_spec_version_block_number.unwrap_or(u64::MAX));
        Ok((
            if min == 0 { None } else { Some(min) },
            if max == u64::MAX { None } else { Some(max) },
        ))
    }
}

use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::TraceRow;

pub(crate) trait CrystalTraceAPIPostgreSQLStorage {
    async fn get_trace_count(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        key_prefix: Option<&[u8]>,
        key_params: Option<&[u8]>,
    ) -> anyhow::Result<u64>;
    async fn get_traces(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        key_prefix: Option<&[u8]>,
        key_params: Option<&[u8]>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>>;
    async fn get_trace_count_by_block_hash(&self, block_hash: &[u8]) -> anyhow::Result<u64>;
    async fn get_traces_by_block_hash(
        &self,
        block_hash: &[u8],
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>>;
    async fn get_trace_count_by_block_number(&self, block_number: u64) -> anyhow::Result<u64>;
    async fn get_traces_by_block_number(
        &self,
        block_number: u64,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>>;
}

impl CrystalTraceAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_trace_count(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        key_prefix: Option<&[u8]>,
        key_params: Option<&[u8]>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM trace
            WHERE
                ($1 IS NULL OR block_number >= $1)
                AND ($2 IS NULL OR block_number <= $2)
                AND ($3 IS NULL OR key_prefix = $3)
                AND ($4 IS NULL OR key_params = $4)
            "#,
        )
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(key_prefix)
        .bind(key_params)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_traces(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        key_prefix: Option<&[u8]>,
        key_params: Option<&[u8]>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<TraceRow> = sqlx::query_as(
            r#"
            SELECT
                T.id, T.block_hash, T.block_number, T.spec_version, T.index,
                T.key_prefix, T.key_params, T.value, T.ext_id, T.method, T.parent_id,
                T.metadata_storage_item_id, T.is_known_key
            FROM trace T
            WHERE
                ($1 IS NULL OR block_number >= $1)
                AND ($2 IS NULL OR block_number <= $2)
                AND ($3 IS NULL OR key_prefix = $3)
                AND ($4 IS NULL OR key_params = $4)
            ORDER BY T.block_number DESC, T.index ASC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(key_prefix)
        .bind(key_params)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    async fn get_trace_count_by_block_hash(&self, block_hash: &[u8]) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM trace
            WHERE
                ($1 IS NULL OR block_hash = $1)
            "#,
        )
        .bind(block_hash)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_traces_by_block_hash(
        &self,
        block_hash: &[u8],
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<TraceRow> = sqlx::query_as(
            r#"
            SELECT
                T.id, T.block_hash, T.block_number, T.spec_version, T.index,
                T.key_prefix, T.key_params, T.value, T.ext_id, T.method, T.parent_id,
                T.metadata_storage_item_id, T.is_known_key
            FROM trace T
            WHERE
                ($1 IS NULL OR T.block_hash = $1)
            ORDER BY T.index ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(block_hash)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    async fn get_trace_count_by_block_number(&self, block_number: u64) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM trace
            WHERE
                ($1 IS NULL OR block_number = $1)
            "#,
        )
        .bind(block_number as i64)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_traces_by_block_number(
        &self,
        block_number: u64,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<TraceRow> = sqlx::query_as(
            r#"
            SELECT
                T.id, T.block_hash, T.block_number, T.spec_version, T.index,
                T.key_prefix, T.key_params, T.value, T.ext_id, T.method, T.parent_id,
                T.metadata_storage_item_id, T.is_known_key
            FROM trace T
            WHERE
                ($1 IS NULL OR T.block_number = $1)
            ORDER BY T.index ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(block_number as i64)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }
}

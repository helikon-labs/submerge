use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::TraceRow;

pub(crate) trait CrystalTraceAPIPostgreSQLStorage {
    async fn get_trace_count(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        key_prefix: Option<&[u8]>,
        key_params: Option<&[u8]>,
    ) -> anyhow::Result<u64>;
    async fn get_traces(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
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
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        key_prefix: Option<&[u8]>,
        key_params: Option<&[u8]>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT COUNT(*)
            FROM trace
            WHERE 1=1
            "#,
        );
        if let Some(key_prefix) = key_prefix {
            query_builder
                .push(" AND key_prefix = ")
                .push_bind(key_prefix);
        }
        if let Some(key_params) = key_params {
            query_builder
                .push(" AND key_params = ")
                .push_bind(key_params);
        }
        if let Some(min) = min_block_number {
            query_builder.push(" AND block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND block_number <= ").push_bind(max);
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_traces(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        key_prefix: Option<&[u8]>,
        key_params: Option<&[u8]>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                id, block_hash, block_number, spec_version, index,
                key_prefix, key_params, value, ext_id, method, parent_id,
                metadata_storage_item_id, is_known_key
            FROM trace
            WHERE 1=1
            "#,
        );
        if let Some(key_prefix) = key_prefix {
            query_builder
                .push(" AND key_prefix = ")
                .push_bind(key_prefix);
        }
        if let Some(key_params) = key_params {
            query_builder
                .push(" AND key_params = ")
                .push_bind(key_params);
        }
        if let Some(min) = min_block_number {
            query_builder.push(" AND block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND block_number <= ").push_bind(max);
        }
        query_builder.push(" ORDER BY block_number DESC, index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<TraceRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_trace_count_by_block_hash(&self, block_hash: &[u8]) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM trace
            WHERE block_hash = $1
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
                id, block_hash, block_number, spec_version, index,
                key_prefix, key_params, value, ext_id, method, parent_id,
                metadata_storage_item_id, is_known_key
            FROM trace T
            WHERE block_hash = $1
            ORDER BY index ASC
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
            WHERE block_number = $1
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
                id, block_hash, block_number, spec_version, index,
                key_prefix, key_params, value, ext_id, method, parent_id,
                metadata_storage_item_id, is_known_key
            FROM trace
            WHERE block_number = $1
            ORDER BY index ASC
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

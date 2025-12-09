use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::TraceRow;

const COUNT: &str = r#"
    SELECT COUNT(*)
    FROM trace T
"#;
const SELECT: &str = r#"
    SELECT
        T.id, T.block_hash, T.block_number, T.spec_version, T.index,
        T.key_prefix, T.key_params, T.value, T.ext_id, T.method, T.parent_id, T.is_known_key,
        MP.index AS pallet_index, MP.name AS pallet_name,
        MSI.index AS pallet_storage_item_index, MSI.name AS pallet_storage_item_name
    FROM trace T
    LEFT JOIN metadata_storage_item MSI ON T.metadata_storage_item_id = MSI.id
    LEFT JOIN metadata_pallet MP ON MSI.pallet_id = MP.id
"#;

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
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<TraceRow>>;
    async fn get_trace_count_by_block_hash(&self, block_hash: &[u8]) -> anyhow::Result<u64>;
    async fn get_traces_by_block_hash(
        &self,
        block_hash: &[u8],
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<TraceRow>>;
    async fn get_trace_count_by_block_number(&self, block_number: u64) -> anyhow::Result<u64>;
    async fn get_traces_by_block_number(
        &self,
        block_number: u64,
        page: u32,
        page_size: u32,
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
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{COUNT} WHERE 1=1"));
        if let Some(key_prefix) = key_prefix {
            query_builder
                .push(" AND T.key_prefix = ")
                .push_bind(key_prefix);
        }
        if let Some(key_params) = key_params {
            query_builder
                .push(" AND T.key_params = ")
                .push_bind(key_params);
        }
        if let Some(min) = min_block_number {
            query_builder.push(" AND T.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND T.block_number <= ").push_bind(max);
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
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{SELECT} WHERE 1=1"));
        if let Some(key_prefix) = key_prefix {
            query_builder
                .push(" AND T.key_prefix = ")
                .push_bind(key_prefix);
        }
        if let Some(key_params) = key_params {
            query_builder
                .push(" AND T.key_params = ")
                .push_bind(key_params);
        }
        if let Some(min) = min_block_number {
            query_builder.push(" AND T.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND T.block_number <= ").push_bind(max);
        }
        query_builder.push(" ORDER BY T.block_number DESC, T.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<TraceRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_trace_count_by_block_hash(&self, block_hash: &[u8]) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE T.block_hash = ")
            .push_bind(block_hash);
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_traces_by_block_hash(
        &self,
        block_hash: &[u8],
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE T.block_hash = ")
            .push_bind(block_hash);
        query_builder.push(" ORDER BY T.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<TraceRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_trace_count_by_block_number(&self, block_number: u64) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE T.block_number = ")
            .push_bind(block_number as i64);
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_traces_by_block_number(
        &self,
        block_number: u64,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE T.block_number = ")
            .push_bind(block_number as i64);
        query_builder.push(" ORDER BY T.block_hash ASC, T.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<TraceRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }
}

use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::{api::dto::block::BlockQuery, persistence::BlockRow};

fn push_query_params<'a>(query: &'a BlockQuery, query_builder: &mut QueryBuilder<'a, Postgres>) {
    if let Some(status) = &query.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }
    if let Some(min_block_number) = query.min_block_number {
        query_builder.push(" AND number >= ");
        query_builder.push_bind(min_block_number as i64);
    }
    if let Some(max_block_number) = query.max_block_number {
        query_builder.push(" AND number <= ");
        query_builder.push_bind(max_block_number as i64);
    }
    if let Some(min_block_timestamp) = query.min_block_timestamp {
        query_builder.push(" AND timestamp >= ");
        query_builder.push_bind(min_block_timestamp as i64);
    }
    if let Some(max_block_timestamp) = query.max_block_timestamp {
        query_builder.push(" AND timestamp <= ");
        query_builder.push_bind(max_block_timestamp as i64);
    }
    if let Some(min_spec_version) = query.min_spec_version {
        query_builder.push(" AND spec_version >= ");
        query_builder.push_bind(min_spec_version as i64);
    }
    if let Some(max_spec_version) = query.max_spec_version {
        query_builder.push(" AND spec_version <= ");
        query_builder.push_bind(max_spec_version as i64);
    }
    if let Some(author) = query.author {
        query_builder.push(" AND author_account_id = ");
        query_builder.push_bind(author.bytes());
    }
}

pub(crate) trait CrystalBlockAPIPostgreSQLStorage {
    async fn get_block_count(&self, query: &BlockQuery) -> anyhow::Result<u64>;
    async fn get_block_rows(
        &self,
        page: u64,
        page_size: u64,
        query: &BlockQuery,
    ) -> anyhow::Result<Vec<BlockRow>>;
}

impl CrystalBlockAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_block_count(&self, query: &BlockQuery) -> anyhow::Result<u64> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COUNT(*)
            FROM block WHERE 1=1
            "#,
        );
        push_query_params(query, &mut query_builder);

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count.0 as u64)
    }

    async fn get_block_rows(
        &self,
        page: u64,
        page_size: u64,
        query: &BlockQuery,
    ) -> anyhow::Result<Vec<BlockRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_account_id
            FROM block WHERE 1=1
            "#,
        );
        push_query_params(query, &mut query_builder);

        query_builder.push(" ORDER BY number DESC LIMIT ");
        query_builder.push_bind(page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let rows: Vec<BlockRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }
}

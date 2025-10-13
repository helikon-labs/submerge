use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::{api::dto::event::BlockEventQuery, persistence::EventRow};

pub(crate) trait CrystalEventAPIPostgreSQLStorage {
    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_hash(
        &self,
        page: u64,
        page_size: u64,
        block_hash: &[u8],
        query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventRow>>;
    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_number(
        &self,
        page: u64,
        page_size: u64,
        block_number: u64,
        query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventRow>>;
}

impl CrystalEventAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        _query: &BlockEventQuery,
    ) -> anyhow::Result<u64> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COUNT(*)
            FROM event
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_hash = ");
        query_builder.push_bind(block_hash);

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count.0 as u64)
    }

    async fn get_events_by_block_hash(
        &self,
        page: u64,
        page_size: u64,
        block_hash: &[u8],
        _query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM event
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_hash = ");
        query_builder.push_bind(block_hash);

        query_builder.push(" ORDER BY block_number DESC, index ASC LIMIT ");
        query_builder.push_bind(page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let rows: Vec<EventRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        _query: &BlockEventQuery,
    ) -> anyhow::Result<u64> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COUNT(*)
            FROM event
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_number = ");
        query_builder.push_bind(block_number as i64);

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count.0 as u64)
    }

    async fn get_events_by_block_number(
        &self,
        page: u64,
        page_size: u64,
        block_number: u64,
        _query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM event
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_number = ");
        query_builder.push_bind(block_number as i64);

        query_builder.push(" ORDER BY block_number DESC, index ASC LIMIT ");
        query_builder.push_bind(page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let rows: Vec<EventRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }
}

use parity_scale_codec::Encode;
use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::api::dto::event::BlockEventQuery;

pub(crate) trait CrystalEventAPIPostgreSQLStorage {
    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64>;
    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64>;
}

impl CrystalEventAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        query: &BlockEventQuery,
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
}
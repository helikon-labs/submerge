use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::TraceRow;

pub(crate) trait CrystalTraceAPIPostgreSQLStorage {
    async fn get_trace_count(
        &self,
        key: Option<&[u8]>,
        key_prefix: Option<&[u8]>,
    ) -> anyhow::Result<u64>;
    async fn get_traces(
        &self,
        key: Option<&[u8]>,
        key_prefix: Option<&[u8]>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>>;
}

impl CrystalTraceAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_trace_count(
        &self,
        key: Option<&[u8]>,
        key_prefix: Option<&[u8]>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM trace
            WHERE
                ($1 IS NULL OR key = $1)
                AND ($2 IS NULL OR key_prefix = $2)
            "#,
        )
        .bind(key)
        .bind(key_prefix)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_traces(
        &self,
        key: Option<&[u8]>,
        key_prefix: Option<&[u8]>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<TraceRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<TraceRow> = sqlx::query_as(
            r#"
            SELECT
                T.id, T.block_hash, T.block_number, T.spec_version, T.index,
                T.key, T.key_prefix, T.value, T.ext_id, T.method, T.parent_id
            FROM trace T
            WHERE
                ($1 IS NULL OR T.key = $1)
                AND ($2 IS NULL OR T.key_prefix = $2)
            ORDER BY T.block_number DESC, T.index ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(key)
        .bind(key_prefix)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }
}

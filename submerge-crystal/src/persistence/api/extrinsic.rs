use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::{
    api::dto::extrinsic::{BlockExtrinsicQuery, ExtrinsicQuery},
    persistence::ExtrinsicRow,
};

fn push_extrinsic_query_params<'a>(
    query: &'a ExtrinsicQuery,
    query_builder: &mut QueryBuilder<'a, Postgres>,
) {
    if let Some(min_block_number) = query.min_block_number {
        query_builder.push(" AND block_number >= ");
        query_builder.push_bind(min_block_number as i64);
    }
    if let Some(max_block_number) = query.max_block_number {
        query_builder.push(" AND block_number <= ");
        query_builder.push_bind(max_block_number as i64);
    }
    if let Some(min_block_timestamp) = query.min_block_timestamp {
        query_builder.push(" AND block_timestamp >= ");
        query_builder.push_bind(min_block_timestamp as i64);
    }
    if let Some(max_block_timestamp) = query.max_block_timestamp {
        query_builder.push(" AND block_timestamp <= ");
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
    if query.is_signed.unwrap_or(false) {
        query_builder.push(" AND signature IS NOT NULL ");
    }
}

pub(crate) trait CrystalExtrinsicAPIPostgreSQLStorage {
    async fn get_extrinsic_count(&self, query: &ExtrinsicQuery) -> anyhow::Result<u64>;
    async fn get_extrinsic_rows(
        &self,
        page_number: u64,
        page_size: u64,
        query: &ExtrinsicQuery,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_row_count_by_block_hash(
        &self,
        block_hash: &[u8],
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsic_rows_by_block_hash(
        &self,
        page_number: u64,
        page_size: u64,
        block_hash: &[u8],
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_row_count_by_block_number(
        &self,
        block_number: u64,
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsic_rows_by_block_number(
        &self,
        page_number: u64,
        page_size: u64,
        block_number: u64,
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_rows_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_row_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<ExtrinsicRow>>;
}

impl CrystalExtrinsicAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_extrinsic_count(&self, query: &ExtrinsicQuery) -> anyhow::Result<u64> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COUNT(*)
            FROM extrinsic WHERE 1=1
            "#,
        );
        push_extrinsic_query_params(query, &mut query_builder);

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count.0 as u64)
    }

    async fn get_extrinsic_rows(
        &self,
        page_number: u64,
        page_size: u64,
        query: &ExtrinsicQuery,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page_number - 1) * page_size;
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic WHERE 1=1
            "#,
        );
        push_extrinsic_query_params(query, &mut query_builder);

        query_builder.push(" ORDER BY block_number DESC, index ASC LIMIT ");
        query_builder.push_bind(page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let rows: Vec<ExtrinsicRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_extrinsic_row_count_by_block_hash(
        &self,
        block_hash: &[u8],
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<u64> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_hash = ");
        query_builder.push_bind(block_hash);
        if query.is_signed.unwrap_or(false) {
            query_builder.push(" AND signature IS NOT NULL ");
        }

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count.0 as u64)
    }

    async fn get_extrinsic_rows_by_block_hash(
        &self,
        page_number: u64,
        page_size: u64,
        block_hash: &[u8],
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page_number - 1) * page_size;
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_hash = ");
        query_builder.push_bind(block_hash);
        if query.is_signed.unwrap_or(false) {
            query_builder.push(" AND signature IS NOT NULL ");
        }

        query_builder.push(" ORDER BY block_number DESC, index ASC LIMIT ");
        query_builder.push_bind(page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let rows: Vec<ExtrinsicRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_extrinsic_row_count_by_block_number(
        &self,
        block_number: u64,
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<u64> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_number = ");
        query_builder.push_bind(block_number as i64);
        if query.is_signed.unwrap_or(false) {
            query_builder.push(" AND signature IS NOT NULL ");
        }

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count.0 as u64)
    }

    async fn get_extrinsic_rows_by_block_number(
        &self,
        page_number: u64,
        page_size: u64,
        block_number: u64,
        query: &BlockExtrinsicQuery,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page_number - 1) * page_size;
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic
            WHERE 1=1
            "#,
        );
        query_builder.push(" AND block_number = ");
        query_builder.push_bind(block_number as i64);
        if query.is_signed.unwrap_or(false) {
            query_builder.push(" AND signature IS NOT NULL ");
        }

        query_builder.push(" ORDER BY block_number DESC, index ASC LIMIT ");
        query_builder.push_bind(page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let rows: Vec<ExtrinsicRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_extrinsic_rows_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let rows: Vec<ExtrinsicRow>  = sqlx::query_as(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic
            WHERE block_number = $1 AND index = $2
            "#,
        )
        .bind(block_number as i64)
        .bind(index as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    async fn get_extrinsic_row_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<ExtrinsicRow>> {
        let row: Option<ExtrinsicRow>  = sqlx::query_as(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic
            WHERE block_hash = $1 AND index = $2
            "#,
        )
        .bind(block_hash)
        .bind(index as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(row)
    }
}

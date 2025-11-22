use parity_scale_codec::Encode;
use sqlx::{Postgres, QueryBuilder};
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::ExtrinsicRow;

pub(crate) trait CrystalExtrinsicAPIPostgreSQLStorage {
    async fn get_extrinsic_count(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsics(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_count_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsics_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_count_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsics_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsics_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<ExtrinsicRow>>;
    async fn get_extrinsic_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<ExtrinsicRow>>;
}

impl CrystalExtrinsicAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_extrinsic_count(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE 1=1
            "#,
        );
        if let Some(min) = min_block_number {
            query_builder.push(" AND block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND block_number <= ").push_bind(max);
        }
        if let Some(is_signed) = is_signed {
            if is_signed {
                query_builder.push(" AND multi_signature IS NOT NULL");
            } else {
                query_builder.push(" AND multi_signature IS NULL");
            }
        }
        if let Some(addr) = signer_multi_address {
            query_builder
                .push(" AND signer_multi_address = ")
                .push_bind(addr.encode());
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_extrinsics(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index,
                hash, index, version, signer_multi_address, multi_signature, extra, is_successful
            FROM extrinsic
            WHERE 1=1
            "#,
        );
        if let Some(min) = min_block_number {
            query_builder.push(" AND block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND block_number <= ").push_bind(max);
        }
        if let Some(is_signed) = is_signed {
            if is_signed {
                query_builder.push(" AND multi_signature IS NOT NULL");
            } else {
                query_builder.push(" AND multi_signature IS NULL");
            }
        }
        if let Some(addr) = signer_multi_address {
            query_builder
                .push(" AND signer_multi_address = ")
                .push_bind(addr.encode());
        }
        query_builder.push(" ORDER BY block_number DESC, index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<ExtrinsicRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_extrinsic_count_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE
                block_hash = $1
                AND ($2 IS NULL OR (($2 AND multi_signature IS NOT NULL) OR (NOT $2 AND multi_signature IS NULL)))
                AND ($3 IS NULL OR signer_multi_address = $3)
            "#,
        )
        .bind(block_hash)
        .bind(is_signed)
        .bind(signer_multi_address.as_ref().map(|signer_multi_address| signer_multi_address.encode()))
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_extrinsics_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT
                id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index,
                hash, index, version, signer_multi_address, multi_signature, extra, is_successful
            FROM extrinsic
            WHERE
                block_hash = $1
                AND ($2 IS NULL OR (($2 AND multi_signature IS NOT NULL) OR (NOT $2 AND multi_signature IS NULL)))
                AND ($3 IS NULL OR signer_multi_address = $3)
            ORDER BY block_number DESC, index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(block_hash)
        .bind(is_signed)
        .bind(signer_multi_address.as_ref().map(|signer_multi_address| signer_multi_address.encode()))
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    async fn get_extrinsic_count_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE
                block_number = $1
                AND ($2 IS NULL OR (($2 AND multi_signature IS NOT NULL) OR (NOT $2 AND multi_signature IS NULL)))
                AND ($3 IS NULL OR signer_multi_address = $3)
            "#,
        )
        .bind(block_number as i64)
        .bind(is_signed)
        .bind(signer_multi_address.as_ref().map(|signer_multi_address| signer_multi_address.encode()))
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_extrinsics_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT
                id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash,
                index, version, signer_multi_address, multi_signature, extra, is_successful
            FROM extrinsic
            WHERE
                block_number = $1
                AND ($2 IS NULL OR (($2 AND multi_signature IS NOT NULL) OR (NOT $2 AND multi_signature IS NULL)))
                AND ($3 IS NULL OR signer_multi_address = $3)
            ORDER BY block_number DESC, index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(block_number as i64)
        .bind(is_signed)
        .bind(signer_multi_address.as_ref().map(|signer_multi_address| signer_multi_address.encode()))
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    async fn get_extrinsics_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let rows: Vec<ExtrinsicRow>  = sqlx::query_as(
            r#"
            SELECT
                id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index,
                hash, index, version, signer_multi_address, multi_signature, extra, is_successful
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

    async fn get_extrinsic_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<ExtrinsicRow>> {
        let row: Option<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT
                id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index,
                hash, index, version, signer_multi_address, multi_signature, extra, is_successful
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

    async fn get_extrinsic_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<ExtrinsicRow>> {
        let row: Option<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT
                id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index,
                hash, index, version, signer_multi_address, multi_signature, extra, is_successful
            FROM extrinsic
            WHERE hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(row)
    }
}

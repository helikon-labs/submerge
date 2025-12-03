use parity_scale_codec::Encode;
use sqlx::{Postgres, QueryBuilder};
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::ExtrinsicRow;

const COUNT: &str = r#"
    SELECT COUNT(*)
    FROM extrinsic
"#;
const SELECT: &str = r#"
    SELECT
        block_hash, block_number, block_timestamp, spec_version, block_status, trace_index,
        hash, index, version, signer_multi_address, multi_signature, extra, is_successful
    FROM extrinsic
"#;

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
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!(r"{COUNT} WHERE 1=1"));
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
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{SELECT} WHERE 1=1"));
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
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE block_hash = ")
            .push_bind(block_hash);
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

    async fn get_extrinsics_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE block_hash = ")
            .push_bind(block_hash);
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

    async fn get_extrinsic_count_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE block_number = ")
            .push_bind(block_number as i64);
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

    async fn get_extrinsics_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE block_number = ")
            .push_bind(block_number as i64);
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

    async fn get_extrinsics_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE block_number = ")
            .push_bind(block_number as i64);
        query_builder.push(" AND index = ").push_bind(index as i32);

        let rows: Vec<ExtrinsicRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_extrinsic_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<ExtrinsicRow>> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE block_hash = ")
            .push_bind(block_hash);
        query_builder.push(" AND index = ").push_bind(index as i32);

        let row: Option<ExtrinsicRow> = query_builder
            .build_query_as()
            .fetch_optional(&self.connection_pool)
            .await?;
        Ok(row)
    }

    async fn get_extrinsic_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<ExtrinsicRow>> {
        let row: Option<ExtrinsicRow> =
            sqlx::query_as(format!("{SELECT} WHERE hash = $1").as_str())
                .bind(hash)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(row)
    }
}

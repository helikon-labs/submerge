use parity_scale_codec::Encode;
use sqlx::{Postgres, QueryBuilder};
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::{
    persistence::api::call::CrystalCallAPIPostgreSQLStorage,
    types::{api::dto::response::extrinsic::ExtrinsicCursorPosition, persistence::ExtrinsicRow},
};

const COUNT: &str = r#"
    SELECT COUNT(*)
    FROM extrinsic E
"#;
const SELECT: &str = r#"
    SELECT
        E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status, E.trace_index,
        E.hash, E.index, E.version, E.signer_multi_address, E.multi_signature, E.extra, E.is_successful
    FROM extrinsic E
"#;

pub(crate) trait CrystalExtrinsicAPIPostgreSQLStorage {
    async fn get_extrinsics(
        &self,
        cursor_position: Option<ExtrinsicCursorPosition>,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page_size: u32,
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
        page: u32,
        page_size: u32,
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
        page: u32,
        page_size: u32,
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
    async fn get_call_extrinsic_by_hash(
        &self,
        call_hash: &[u8],
    ) -> anyhow::Result<Option<ExtrinsicRow>>;
}

impl CrystalExtrinsicAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_extrinsics(
        &self,
        cursor_position: Option<ExtrinsicCursorPosition>,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        is_signed: Option<bool>,
        signer_multi_address: &Option<MultiAddress>,
        page_size: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{SELECT} WHERE 1=1"));
        if let Some(min) = min_block_number {
            query_builder.push(" AND E.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND E.block_number <= ").push_bind(max);
        }

        if let Some(cursor_position) = cursor_position {
            let block_hash = cursor_position.get_block_hash()?;
            query_builder.push(" AND (");
            query_builder
                .push("E.block_number < ")
                .push_bind(cursor_position.block_number as i64);
            query_builder
                .push(" OR (E.block_number = ")
                .push_bind(cursor_position.block_number as i64);
            query_builder
                .push(" AND E.block_hash > ")
                .push_bind(block_hash.clone());
            query_builder.push(")");
            query_builder
                .push(" OR (E.block_number = ")
                .push_bind(cursor_position.block_number as i64);
            query_builder
                .push(" AND E.block_hash = ")
                .push_bind(block_hash.clone());
            query_builder
                .push(" AND E.index > ")
                .push_bind(cursor_position.index as i32);
            query_builder.push(")");
            query_builder.push(")");
        }

        if let Some(is_signed) = is_signed {
            if is_signed {
                query_builder.push(" AND E.multi_signature IS NOT NULL");
            } else {
                query_builder.push(" AND E.multi_signature IS NULL");
            }
        }
        if let Some(addr) = signer_multi_address {
            query_builder
                .push(" AND E.signer_multi_address = ")
                .push_bind(addr.encode());
        }
        query_builder.push(" ORDER BY E.block_number DESC, E.block_hash ASC, E.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);

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
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        if let Some(is_signed) = is_signed {
            if is_signed {
                query_builder.push(" AND E.multi_signature IS NOT NULL");
            } else {
                query_builder.push(" AND E.multi_signature IS NULL");
            }
        }
        if let Some(addr) = signer_multi_address {
            query_builder
                .push(" AND E.signer_multi_address = ")
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
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        if let Some(is_signed) = is_signed {
            if is_signed {
                query_builder.push(" AND E.multi_signature IS NOT NULL");
            } else {
                query_builder.push(" AND E.multi_signature IS NULL");
            }
        }
        if let Some(addr) = signer_multi_address {
            query_builder
                .push(" AND E.signer_multi_address = ")
                .push_bind(addr.encode());
        }
        query_builder.push(" ORDER BY E.index ASC");
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
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        if let Some(is_signed) = is_signed {
            if is_signed {
                query_builder.push(" AND E.multi_signature IS NOT NULL");
            } else {
                query_builder.push(" AND E.multi_signature IS NULL");
            }
        }
        if let Some(addr) = signer_multi_address {
            query_builder
                .push(" AND E.signer_multi_address = ")
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
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        if let Some(is_signed) = is_signed {
            if is_signed {
                query_builder.push(" AND E.multi_signature IS NOT NULL");
            } else {
                query_builder.push(" AND E.multi_signature IS NULL");
            }
        }
        if let Some(addr) = signer_multi_address {
            query_builder
                .push(" AND E.signer_multi_address = ")
                .push_bind(addr.encode());
        }
        query_builder.push(" ORDER BY E.block_number DESC, E.block_hash ASC, E.index ASC");
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
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        query_builder
            .push(" AND E.index = ")
            .push_bind(index as i32);

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
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        query_builder
            .push(" AND E.index = ")
            .push_bind(index as i32);

        let row: Option<ExtrinsicRow> = query_builder
            .build_query_as()
            .fetch_optional(&self.connection_pool)
            .await?;
        Ok(row)
    }

    async fn get_extrinsic_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<ExtrinsicRow>> {
        let row: Option<ExtrinsicRow> =
            sqlx::query_as(format!("{SELECT} WHERE E.hash = $1").as_str())
                .bind(hash)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(row)
    }

    async fn get_call_extrinsic_by_hash(
        &self,
        call_hash: &[u8],
    ) -> anyhow::Result<Option<ExtrinsicRow>> {
        let Some(call) = self.get_call_by_hash(call_hash, false).await? else {
            return Ok(None);
        };
        let row: Option<ExtrinsicRow> =
            sqlx::query_as(format!("{SELECT} WHERE hash = $1").as_str())
                .bind(call.extrinsic_hash)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(row)
    }
}

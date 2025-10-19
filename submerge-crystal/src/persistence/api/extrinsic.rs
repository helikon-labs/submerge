use parity_scale_codec::Encode;
use submerge_base::types::substrate::{account_id::AccountId, multi_address::MultiAddress};
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::ExtrinsicRow;

pub(crate) trait CrystalExtrinsicAPIPostgreSQLStorage {
    async fn get_extrinsic_count(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        is_signed: Option<bool>,
        signer: Option<AccountId>,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsics(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        is_signed: Option<bool>,
        signer: Option<AccountId>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_count_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer: Option<AccountId>,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsics_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer: Option<AccountId>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>>;
    async fn get_extrinsic_count_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer: Option<AccountId>,
    ) -> anyhow::Result<u64>;
    async fn get_extrinsics_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer: Option<AccountId>,
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
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        is_signed: Option<bool>,
        signer: Option<AccountId>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE
                ($1 IS NULL OR block_number >= $1)
                AND ($2 IS NULL OR block_number <= $2)
                AND ($3 IS NULL OR block_timestamp >= $3)
                AND ($4 IS NULL OR block_timestamp <= $4)
                AND ($5 IS NULL OR spec_version >= $5)
                AND ($6 IS NULL OR spec_version <= $6)
                AND ($7 IS NULL OR (($7 AND signature IS NOT NULL) OR (NOT $7 AND signature IS NULL)))
                AND ($8 IS NULL OR signer = $8)
            "#,
        )
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(min_block_timestamp.map(|n| n as i64))
        .bind(max_block_timestamp.map(|n| n as i64))
        .bind(min_spec_version.map(|n| n as i32))
        .bind(max_spec_version.map(|n| n as i32))
        .bind(is_signed)
        .bind(signer.map(|signer| {
            let signer: MultiAddress = MultiAddress::Id(signer);
            signer.encode()
        }))
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_extrinsics(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        is_signed: Option<bool>,
        signer: Option<AccountId>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic
            WHERE
                ($1 IS NULL OR block_number >= $1)
                AND ($2 IS NULL OR block_number <= $2)
                AND ($3 IS NULL OR block_timestamp >= $3)
                AND ($4 IS NULL OR block_timestamp <= $4)
                AND ($5 IS NULL OR spec_version >= $5)
                AND ($6 IS NULL OR spec_version <= $6)
                AND ($7 IS NULL OR (($7 AND signature IS NOT NULL) OR (NOT $7 AND signature IS NULL)))
                AND ($8 IS NULL OR signer = $8)
            ORDER BY block_number DESC, index ASC
            LIMIT $9 OFFSET $10
            "#,
        )
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(min_block_timestamp.map(|n| n as i64))
        .bind(max_block_timestamp.map(|n| n as i64))
        .bind(min_spec_version.map(|n| n as i32))
        .bind(max_spec_version.map(|n| n as i32))
        .bind(is_signed)
        .bind(signer.map(|signer| {
            let signer: MultiAddress = MultiAddress::Id(signer);
            signer.encode()
        }))
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    async fn get_extrinsic_count_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer: Option<AccountId>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE
                block_hash = $1
                AND ($2 IS NULL OR (($2 AND signature IS NOT NULL) OR (NOT $2 AND signature IS NULL)))
                AND ($3 IS NULL OR signer = $3)
            "#,
        )
        .bind(block_hash)
        .bind(is_signed)
        .bind(signer.map(|signer| {
            let signer: MultiAddress = MultiAddress::Id(signer);
            signer.encode()
        }))
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_extrinsics_by_block_hash(
        &self,
        block_hash: &[u8],
        is_signed: Option<bool>,
        signer: Option<AccountId>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic
            WHERE
                block_hash = $1
                AND ($2 IS NULL OR (($2 AND signature IS NOT NULL) OR (NOT $2 AND signature IS NULL)))
                AND ($3 IS NULL OR signer = $3)
            ORDER BY block_number DESC, index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(block_hash)
        .bind(is_signed)
        .bind(signer.map(|signer| {
            let signer: MultiAddress = MultiAddress::Id(signer);
            signer.encode()
        }))
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
        signer: Option<AccountId>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM extrinsic
            WHERE
                block_number = $1
                AND ($2 IS NULL OR (($2 AND signature IS NOT NULL) OR (NOT $2 AND signature IS NULL)))
                AND ($3 IS NULL OR signer = $3)
            "#,
        )
        .bind(block_number as i64)
        .bind(is_signed)
        .bind(signer.map(|signer| {
            let signer: MultiAddress = MultiAddress::Id(signer);
            signer.encode()
        }))
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_extrinsics_by_block_number(
        &self,
        block_number: u64,
        is_signed: Option<bool>,
        signer: Option<AccountId>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<ExtrinsicRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
            FROM extrinsic
            WHERE
                block_number = $1
                AND ($2 IS NULL OR (($2 AND signature IS NOT NULL) OR (NOT $2 AND signature IS NULL)))
                AND ($3 IS NULL OR signer = $3)
            ORDER BY block_number DESC, index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(block_number as i64)
        .bind(is_signed)
        .bind(signer.map(|signer| {
            let signer: MultiAddress = MultiAddress::Id(signer);
            signer.encode()
        }))
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

    async fn get_extrinsic_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<ExtrinsicRow>> {
        let row: Option<ExtrinsicRow> = sqlx::query_as(
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

    async fn get_extrinsic_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<ExtrinsicRow>> {
        let row: Option<ExtrinsicRow> = sqlx::query_as(
            r#"
            SELECT id, block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful
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

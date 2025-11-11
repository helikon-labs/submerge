use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::persistence::CallRow;

pub(crate) trait CrystalCallAPIPostgreSQLStorage {
    async fn get_call_count(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<CallRow>>;
}

impl CrystalCallAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_call_count(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR C.block_number >= $1)
                AND ($2 IS NULL OR C.block_number <= $2)
                AND ($3 IS NULL OR C.block_timestamp >= $3)
                AND ($4 IS NULL OR C.block_timestamp <= $4)
                AND ($5 IS NULL OR C.spec_version >= $5)
                AND ($6 IS NULL OR C.spec_version <= $6)
                AND ($7 IS NULL OR MP.name ILIKE '%' || $7 || '%')
                AND ($8 IS NULL OR MC.name ILIKE '%' || $8 || '%')
            "#,
        )
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(min_block_timestamp.map(|n| n as i64))
        .bind(max_block_timestamp.map(|n| n as i64))
        .bind(min_spec_version.map(|n| n as i32))
        .bind(max_spec_version.map(|n| n as i32))
        .bind(pallet_name)
        .bind(pallet_call_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_calls(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let call_rows: Vec<CallRow> = sqlx::query_as(
            r#"
            SELECT
                C.id, C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_id, C.extrinsic_index, C.extrinsic_hash,
                C.parent_call_id, C.nesting_index, C.args, C.is_successful,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR C.block_number >= $1)
                AND ($2 IS NULL OR C.block_number <= $2)
                AND ($3 IS NULL OR C.block_timestamp >= $3)
                AND ($4 IS NULL OR C.block_timestamp <= $4)
                AND ($5 IS NULL OR C.spec_version >= $5)
                AND ($6 IS NULL OR C.spec_version <= $6)
                AND ($7 IS NULL OR MP.name ILIKE '%' || $7 || '%')
                AND ($8 IS NULL OR MC.name ILIKE '%' || $8 || '%')
            ORDER BY C.block_number DESC, C.extrinsic_index ASC
            LIMIT $9 OFFSET $10
            "#,
        )
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(min_block_timestamp.map(|n| n as i64))
        .bind(max_block_timestamp.map(|n| n as i64))
        .bind(min_spec_version.map(|n| n as i32))
        .bind(max_spec_version.map(|n| n as i32))
        .bind(pallet_name)
        .bind(pallet_call_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(call_rows)
    }

    async fn get_call_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_hash = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR MC.name ILIKE '%' || $3 || '%')
            "#,
        )
        .bind(block_hash)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_calls_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let call_rows: Vec<CallRow> = sqlx::query_as(
            r#"
            SELECT
                C.id, C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_id, C.extrinsic_index, C.extrinsic_hash,
                C.parent_call_id, C.nesting_index, C.args, C.is_successful,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_hash = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR MC.name ILIKE '%' || $3 || '%')
            ORDER BY C.extrinsic_index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(block_hash)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(call_rows)
    }

    async fn get_call_count_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_number = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR MC.name ILIKE '%' || $3 || '%')
            "#,
        )
        .bind(block_number as i64)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_calls_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let call_rows: Vec<CallRow> = sqlx::query_as(
            r#"
            SELECT
                C.id, C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_id, C.extrinsic_index, C.extrinsic_hash,
                C.parent_call_id, C.nesting_index, C.args, C.is_successful,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_number = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR MC.name ILIKE '%' || $3 || '%')
            ORDER BY C.extrinsic_index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(block_number as i64)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(call_rows)
    }

    async fn get_call_count_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_hash = $1 AND C.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR MC.name ILIKE '%' || $4 || '%')
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index as i32)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_calls_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let call_rows: Vec<CallRow> = sqlx::query_as(
            r#"
            SELECT
                C.id, C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_id, C.extrinsic_index, C.extrinsic_hash,
                C.parent_call_id, C.nesting_index, C.args, C.is_successful,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_hash = $1 AND C.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR MC.name ILIKE '%' || $4 || '%')
            ORDER BY C.extrinsic_index ASC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index as i32)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(call_rows)
    }

    async fn get_call_count_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_number = $1 AND C.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR MC.name ILIKE '%' || $4 || '%')
            "#,
        )
        .bind(block_number as i64)
        .bind(extrinsic_index as i32)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_calls_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let call_rows: Vec<CallRow> = sqlx::query_as(
            r#"
            SELECT
                C.id, C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_id, C.extrinsic_index, C.extrinsic_hash,
                C.parent_call_id, C.nesting_index, C.args, C.is_successful,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.block_number = $1 AND C.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR MC.name ILIKE '%' || $4 || '%')
            ORDER BY C.extrinsic_index ASC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(block_number as i64)
        .bind(extrinsic_index as i32)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(call_rows)
    }

    async fn get_call_count_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.extrinsic_hash = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR MC.name ILIKE '%' || $3 || '%')
            "#,
        )
        .bind(extrinsic_hash)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_calls_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let call_rows: Vec<CallRow> = sqlx::query_as(
            r#"
            SELECT
                C.id, C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_id, C.extrinsic_index, C.extrinsic_hash,
                C.parent_call_id, C.nesting_index, C.args, C.is_successful,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE
                C.extrinsic_hash = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR MC.name ILIKE '%' || $3 || '%')
            ORDER BY C.extrinsic_index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(extrinsic_hash)
        .bind(pallet_name)
        .bind(pallet_call_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(call_rows)
    }

    async fn get_call_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<CallRow>> {
        let call_row: Option<CallRow> = sqlx::query_as(
            r#"
            SELECT
                C.id, C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_id, C.extrinsic_index, C.extrinsic_hash,
                C.parent_call_id, C.nesting_index, C.args, C.is_successful,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM call C
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            WHERE C.hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(call_row)
    }
}

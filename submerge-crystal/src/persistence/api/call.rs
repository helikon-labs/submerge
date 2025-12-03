use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::{escape_like_pattern, PostgreSQLStorage};

use crate::types::persistence::CallRow;

const COUNT: &str = r#"
    SELECT COUNT(*)
    FROM call C
    JOIN metadata_call MC ON C.metadata_call_id = MC.id
    JOIN metadata_pallet MP ON MC.pallet_id = MP.id
"#;
const SELECT: &str = r#"
    SELECT
        C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
        C.extrinsic_index, C.extrinsic_hash, C.parent_call_hash, C.call_path, C.call_index,
        C.extrinsic_is_successful, C.args,
        MP.index AS pallet_index, MP.name AS pallet_name,
        MC.index AS pallet_call_index, MC.name AS pallet_call_name
    FROM call C
    JOIN metadata_call MC ON C.metadata_call_id = MC.id
    JOIN metadata_pallet MP ON MC.pallet_id = MP.id
"#;

pub(crate) trait CrystalCallAPIPostgreSQLStorage {
    async fn get_call_count(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
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
    async fn call_exists_by_hash(&self, hash: &[u8]) -> anyhow::Result<bool>;
    async fn get_call_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<CallRow>>;
    async fn get_sub_call_count_by_hash(
        &self,
        hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_sub_calls_by_hash(
        &self,
        hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_parent_call_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<CallRow>>;
}

impl CrystalCallAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_call_count(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{COUNT} WHERE 1=1"));
        if let Some(min) = min_block_number {
            query_builder.push(" AND C.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND C.block_number <= ").push_bind(max);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_calls(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{SELECT} WHERE 1=1"));
        if let Some(min) = min_block_number {
            query_builder.push(" AND C.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND C.block_number <= ").push_bind(max);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        query_builder
            .push(" ORDER BY C.block_number DESC, C.extrinsic_index ASC, C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_call_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.block_hash = ")
            .push_bind(block_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
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
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" AND C.block_hash = ")
            .push_bind(block_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        query_builder.push(" ORDER BY C.extrinsic_index ASC, C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_call_count_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.block_number = ")
            .push_bind(block_number as i64);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
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
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" AND C.block_number = ")
            .push_bind(block_number as i64);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        query_builder.push(" ORDER BY C.block_hash ASC, C.extrinsic_index ASC, C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_call_count_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.block_hash = ")
            .push_bind(block_hash);
        query_builder
            .push(" AND C.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
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
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" AND C.block_hash = ")
            .push_bind(block_hash);
        query_builder
            .push(" AND C.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        query_builder.push(" ORDER BY C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_call_count_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.block_number = ")
            .push_bind(block_number as i64);
        query_builder
            .push(" AND C.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
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
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" AND C.block_number = ")
            .push_bind(block_number as i64);
        query_builder
            .push(" AND C.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        query_builder.push(" ORDER BY C.block_hash ASC, C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_call_count_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.extrinsic_hash = ")
            .push_bind(extrinsic_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
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
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" AND C.extrinsic_hash = ")
            .push_bind(extrinsic_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        query_builder.push(" ORDER BY C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn call_exists_by_hash(&self, hash: &[u8]) -> anyhow::Result<bool> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM call WHERE hash = $1)")
            .bind(hash)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(exists)
    }

    async fn get_call_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<CallRow>> {
        let call_row: Option<CallRow> =
            sqlx::query_as(format!("{SELECT} WHERE C.hash = $1").as_str())
                .bind(hash)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(call_row)
    }

    async fn get_sub_call_count_by_hash(
        &self,
        hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.parent_call_hash = ")
            .push_bind(hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_sub_calls_by_hash(
        &self,
        hash: &[u8],
        pallet_name: &Option<String>,
        pallet_call_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" AND C.parent_call_hash = ")
            .push_bind(hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_call_name) = pallet_call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_call_name)));
        }
        query_builder.push(" ORDER BY C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_parent_call_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<CallRow>> {
        let call_row: Option<CallRow> = sqlx::query_as(
            r#"
            WITH child_info AS (
                SELECT parent_call_hash, block_number
                FROM call
                WHERE hash = $1
                LIMIT 1
            )
            SELECT
                C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
                C.extrinsic_index, C.extrinsic_hash, C.parent_call_hash, C.call_path, C.call_index,
                C.extrinsic_is_successful, C.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                MC.index AS pallet_call_index, MC.name AS pallet_call_name
            FROM child_info
            JOIN call C ON C.hash = child_info.parent_call_hash
                AND C.block_number = child_info.block_number
            JOIN metadata_call MC ON C.metadata_call_id = MC.id
            JOIN metadata_pallet MP ON MC.pallet_id = MP.id
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(call_row)
    }
}

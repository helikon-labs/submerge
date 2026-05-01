use serde_json::Value as JSONValue;
use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::{escape_like_pattern, PostgreSQLStorage};

use crate::types::{api::dto::response::call::CallCursorPosition, persistence::CallRow};

const COUNT: &str = r#"
    SELECT COUNT(*)
    FROM call C
    JOIN metadata_call MC ON C.metadata_call_id = MC.id
    JOIN metadata_pallet MP ON MC.pallet_id = MP.id
"#;

fn get_select_query(include_args: bool) -> String {
    let query = r#"
        SELECT
            C.hash, C.block_hash, C.block_number, C.block_timestamp, C.spec_version, C.block_status,
            C.extrinsic_index, C.extrinsic_hash, C.parent_call_hash, C.call_path, C.call_index,
            C.extrinsic_is_successful, C.extrinsic_is_signed, C.is_successful, {ARGS_PLACEHOLDER},
            MP.index AS pallet_index, MP.name AS pallet_name,
            MC.index AS pallet_call_index, MC.name AS call_name
        FROM call C
        JOIN metadata_call MC ON C.metadata_call_id = MC.id
        JOIN metadata_pallet MP ON MC.pallet_id = MP.id
    "#;
    if include_args {
        query.replace("{ARGS_PLACEHOLDER}", "C.args")
    } else {
        query.replace("{ARGS_PLACEHOLDER}", "NULL::jsonb as args")
    }
}

pub(crate) trait CrystalCallAPIPostgreSQLStorage {
    async fn get_calls(
        &self,
        cursor_position: Option<CallCursorPosition>,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        metadata_call_ids: Option<Vec<u32>>,
        page_size: u32,
        extrinsic_is_signed: Option<bool>,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        extrinsic_is_signed: Option<bool>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        extrinsic_is_signed: Option<bool>,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        extrinsic_is_signed: Option<bool>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        extrinsic_is_signed: Option<bool>,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_call_count_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_calls_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn call_exists_by_hash(&self, hash: &[u8]) -> anyhow::Result<bool>;
    async fn get_call_by_hash(
        &self,
        hash: &[u8],
        include_args: bool,
    ) -> anyhow::Result<Option<CallRow>>;
    async fn get_call_args_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<JSONValue>>;
    async fn get_sub_call_count_by_hash(
        &self,
        hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_sub_calls_by_hash(
        &self,
        hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>>;
    async fn get_parent_call_by_hash(
        &self,
        hash: &[u8],
        include_args: bool,
    ) -> anyhow::Result<Option<CallRow>>;
    async fn get_extrinsic_root_call_by_hash(
        &self,
        extrinsic_hash: &[u8],
        include_args: bool,
    ) -> anyhow::Result<Option<CallRow>>;
}

impl CrystalCallAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_calls(
        &self,
        cursor_position: Option<CallCursorPosition>,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        metadata_call_ids: Option<Vec<u32>>,
        page_size: u32,
        extrinsic_is_signed: Option<bool>,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>> {
        if let Some(metadata_call_ids) = metadata_call_ids.as_deref() {
            if metadata_call_ids.is_empty() {
                return Ok(Vec::new()); // no call matches -> no calls
            }
        }

        let query = get_select_query(false);
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{query} WHERE 1 = 1"));
        if let Some(min) = min_block_number {
            query_builder.push(" AND C.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND C.block_number <= ").push_bind(max);
        }
        if let Some(cursor_position) = cursor_position {
            let block_hash = cursor_position.get_block_hash()?;
            query_builder.push(" AND (");
            query_builder
                .push("C.block_number < ")
                .push_bind(cursor_position.block_number as i64);
            query_builder
                .push(" OR (C.block_number = ")
                .push_bind(cursor_position.block_number as i64);
            query_builder
                .push(" AND C.block_hash > ")
                .push_bind(block_hash.clone());
            query_builder.push(")");
            query_builder
                .push(" OR (C.block_number = ")
                .push_bind(cursor_position.block_number as i64);
            query_builder
                .push(" AND C.block_hash = ")
                .push_bind(block_hash.clone());
            query_builder.push(" AND C.call_index > ").push_bind(
                cursor_position
                    .call_index
                    .iter()
                    .map(|i| *i as i16)
                    .collect::<Vec<i16>>(),
            );
            query_builder.push(")");
            query_builder.push(")");
        }
        if let Some(extrinsic_is_signed) = extrinsic_is_signed {
            query_builder
                .push(" AND C.extrinsic_is_signed = ")
                .push_bind(extrinsic_is_signed);
        }

        if let Some(ids) = &metadata_call_ids {
            query_builder.push(" AND C.metadata_call_id IN (");
            let mut separated = query_builder.separated(", ");
            for id in ids {
                // literal integer, safe because values come from DB, not user text
                separated.push(id.to_string());
            }
            separated.push_unseparated(")");
        }
        query_builder
            .push(" ORDER BY C.block_number DESC, C.block_hash ASC, C.extrinsic_index ASC, C.call_index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);

        let mut rows: Vec<CallRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        if include_args {
            for row in rows.iter_mut() {
                row.args = self.get_call_args_by_hash(&row.hash).await?;
            }
        }
        Ok(rows)
    }

    async fn get_call_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
        extrinsic_is_signed: Option<bool>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.block_hash = ")
            .push_bind(block_hash);
        if let Some(extrinsic_is_signed) = extrinsic_is_signed {
            query_builder
                .push(" AND C.extrinsic_is_signed = ")
                .push_bind(extrinsic_is_signed);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        extrinsic_is_signed: Option<bool>,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let query = get_select_query(include_args);
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(query);
        query_builder
            .push(" AND C.block_hash = ")
            .push_bind(block_hash);
        if let Some(extrinsic_is_signed) = extrinsic_is_signed {
            query_builder
                .push(" AND C.extrinsic_is_signed = ")
                .push_bind(extrinsic_is_signed);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
        extrinsic_is_signed: Option<bool>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE C.block_number = ")
            .push_bind(block_number as i64);
        if let Some(extrinsic_is_signed) = extrinsic_is_signed {
            query_builder
                .push(" AND C.extrinsic_is_signed = ")
                .push_bind(extrinsic_is_signed);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        extrinsic_is_signed: Option<bool>,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let query = get_select_query(include_args);
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(query);
        query_builder
            .push(" AND C.block_number = ")
            .push_bind(block_number as i64);
        if let Some(extrinsic_is_signed) = extrinsic_is_signed {
            query_builder
                .push(" AND C.extrinsic_is_signed = ")
                .push_bind(extrinsic_is_signed);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
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
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let query = get_select_query(include_args);
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(query);
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
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
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
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let query = get_select_query(include_args);
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(query);
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
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
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
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let query = get_select_query(include_args);
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(query);
        query_builder
            .push(" WHERE C.extrinsic_hash = ")
            .push_bind(extrinsic_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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

    async fn get_call_by_hash(
        &self,
        hash: &[u8],
        include_args: bool,
    ) -> anyhow::Result<Option<CallRow>> {
        let query = get_select_query(include_args);
        let call_row: Option<CallRow> =
            sqlx::query_as(format!("{query} WHERE C.hash = $1").as_str())
                .bind(hash)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(call_row)
    }

    async fn get_call_args_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<JSONValue>> {
        let row: Option<(JSONValue,)> = sqlx::query_as("SELECT args FROM call WHERE hash = $1")
            .bind(hash)
            .fetch_optional(&self.connection_pool)
            .await?;
        Ok(row.map(|tuple| tuple.0))
    }

    async fn get_sub_call_count_by_hash(
        &self,
        hash: &[u8],
        pallet_name: &Option<String>,
        call_name: &Option<String>,
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
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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
        call_name: &Option<String>,
        page: u32,
        page_size: u32,
        include_args: bool,
    ) -> anyhow::Result<Vec<CallRow>> {
        let offset = (page - 1) * page_size;
        let query = get_select_query(include_args);
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(query);
        query_builder
            .push(" WHERE C.parent_call_hash = ")
            .push_bind(hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(call_name) = call_name {
            query_builder
                .push(" AND MC.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(call_name)));
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

    async fn get_parent_call_by_hash(
        &self,
        hash: &[u8],
        include_args: bool,
    ) -> anyhow::Result<Option<CallRow>> {
        let Some(child_call) = self.get_call_by_hash(hash, include_args).await? else {
            return Ok(None);
        };
        let Some(parent_call_hash) = child_call.parent_call_hash else {
            return Ok(None);
        };
        self.get_call_by_hash(&parent_call_hash, include_args).await
    }

    async fn get_extrinsic_root_call_by_hash(
        &self,
        extrinsic_hash: &[u8],
        include_args: bool,
    ) -> anyhow::Result<Option<CallRow>> {
        let query = get_select_query(include_args);
        let call_row: Option<CallRow> = sqlx::query_as(
            format!("{query} WHERE C.extrinsic_hash = $1 AND parent_call_hash IS NULL").as_str(),
        )
        .bind(extrinsic_hash)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(call_row)
    }
}

use crate::types::persistence::EventCompositeRow;
use serde_json::Value as JSONValue;
use sqlx::{Postgres, QueryBuilder};
use submerge_persistence::postgres::{escape_like_pattern, PostgreSQLStorage};

const COUNT: &str = r#"
    SELECT COUNT(*)
    FROM event E
    JOIN metadata_event ME ON E.metadata_event_id = ME.id
    JOIN metadata_pallet MP ON ME.pallet_id = MP.id
"#;
const SELECT: &str = r#"
    SELECT
        E.hash, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
        E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index,
        MP.index AS pallet_index, MP.name AS pallet_name,
        ME.index AS pallet_event_index, ME.name AS pallet_event_name
    FROM event E
    JOIN metadata_event ME ON E.metadata_event_id = ME.id
    JOIN metadata_pallet MP ON ME.pallet_id = MP.id
"#;

pub(crate) trait CrystalEventAPIPostgreSQLStorage {
    async fn get_event_count(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_events(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_events_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_event_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<EventCompositeRow>>;
    async fn get_event_count_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_event_count_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_event_count_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_event_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<EventCompositeRow>>;
    async fn get_event_args_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<JSONValue>>;
}

impl CrystalEventAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_event_count(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{COUNT} WHERE 1=1"));
        if let Some(min) = min_block_number {
            query_builder.push(" AND E.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND E.block_number <= ").push_bind(max);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_events(
        &self,
        min_block_number: Option<i64>,
        max_block_number: Option<i64>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("{SELECT} WHERE 1=1"));
        if let Some(min) = min_block_number {
            query_builder.push(" AND E.block_number >= ").push_bind(min);
        }
        if let Some(max) = max_block_number {
            query_builder.push(" AND E.block_number <= ").push_bind(max);
        }
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        query_builder.push(" ORDER BY E.block_number DESC, E.block_hash ASC, E.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        query_builder.push(" ORDER BY index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        query_builder.push(" ORDER BY E.block_hash ASC, E.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_events_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        query_builder
            .push(" AND E.index = ")
            .push_bind(index as i32);
        query_builder.push(" ORDER BY E.block_hash ASC");
        let rows: Vec<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_event_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<EventCompositeRow>> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        query_builder
            .push(" AND E.index = ")
            .push_bind(index as i32);
        let row: Option<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_optional(&self.connection_pool)
            .await?;
        Ok(row)
    }

    async fn get_event_count_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        query_builder
            .push(" AND E.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_number = ")
            .push_bind(block_number as i64);
        query_builder
            .push(" AND E.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        query_builder.push(" ORDER BY E.block_hash ASC, E.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_event_count_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        query_builder
            .push(" AND E.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.block_hash = ")
            .push_bind(block_hash);
        query_builder
            .push(" AND E.extrinsic_index = ")
            .push_bind(extrinsic_index as i32);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        query_builder.push(" ORDER BY E.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_event_count_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(COUNT);
        query_builder
            .push(" WHERE E.extrinsic_hash = ")
            .push_bind(extrinsic_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(count as u64)
    }

    async fn get_events_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(SELECT);
        query_builder
            .push(" WHERE E.extrinsic_hash = ")
            .push_bind(extrinsic_hash);
        if let Some(pallet_name) = pallet_name {
            query_builder
                .push(" AND MP.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_name)));
        }
        if let Some(pallet_event_name) = pallet_event_name {
            query_builder
                .push(" AND ME.name ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(pallet_event_name)));
        }
        query_builder.push(" ORDER BY E.index ASC");
        query_builder.push(" LIMIT ").push_bind(page_size as i64);
        query_builder.push(" OFFSET ").push_bind(offset as i64);

        let rows: Vec<EventCompositeRow> = query_builder
            .build_query_as()
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    async fn get_event_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<EventCompositeRow>> {
        let event_row: Option<EventCompositeRow> =
            sqlx::query_as(format!("{SELECT} WHERE E.hash = $1").as_str())
                .bind(hash)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(event_row)
    }

    async fn get_event_args_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<JSONValue>> {
        let row: Option<(JSONValue,)> = sqlx::query_as("SELECT args FROM event WHERE hash = $1")
            .bind(hash)
            .fetch_optional(&self.connection_pool)
            .await?;
        Ok(row.map(|tuple| tuple.0))
    }
}

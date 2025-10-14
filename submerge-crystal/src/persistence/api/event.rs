use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::{api::dto::event::BlockEventQuery, persistence::EventCompositeRow};

pub(crate) trait CrystalEventAPIPostgreSQLStorage {
    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_hash(
        &self,
        page: u64,
        page_size: u64,
        block_hash: &[u8],
        query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64>;
    async fn get_events_by_block_number(
        &self,
        page: u64,
        page_size: u64,
        block_number: u64,
        query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
}

impl CrystalEventAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            WITH matching_event_ids AS (
                SELECT ME.id
                FROM metadata_event ME
                INNER JOIN metadata_pallet MP ON ME.pallet_id = MP.id
                WHERE MP.name ILIKE $1
                AND ME.name ILIKE $2
            )
            SELECT COUNT(E.*)
            FROM event E
            WHERE E.block_hash = $3
            AND E.metadata_event_id IN (SELECT id FROM matching_event_ids)
            "#,
        )
        .bind(format!("%{}%", query.pallet_name.as_deref().unwrap_or("")))
        .bind(format!("%{}%", query.event_name.as_deref().unwrap_or("")))
        .bind(block_hash)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_hash(
        &self,
        page: u64,
        page_size: u64,
        block_hash: &[u8],
        query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            WITH matching_event_ids AS (
                SELECT ME.id, MP.name AS pallet_name, MP.index AS pallet_index, ME.index AS pallet_event_index, ME.name AS pallet_event_name
                FROM metadata_event ME
                INNER JOIN metadata_pallet MP ON ME.pallet_id = MP.id
                WHERE MP.name ILIKE $1
                AND ME.name ILIKE $2
            )
            SELECT E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status, E.trace_index, M.pallet_index, M.pallet_name, M.pallet_event_index, M.pallet_event_name, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args
            FROM event E
            INNER JOIN matching_event_ids M ON E.metadata_event_id = M.id
            WHERE block_hash = $3
            ORDER BY E.index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
            .bind(format!("%{}%", query.pallet_name.as_deref().unwrap_or("")))
        .bind(format!("%{}%", query.event_name.as_deref().unwrap_or("")))
            .bind(block_hash)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(event_rows)
    }

    async fn get_event_count_by_block_number(
        &self,
        block_number: u64,
        query: &BlockEventQuery,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            WITH matching_event_ids AS (
                SELECT ME.id
                FROM metadata_event ME
                INNER JOIN metadata_pallet MP ON ME.pallet_id = MP.id
                WHERE MP.name ILIKE $1
                AND ME.name ILIKE $2
            )
            SELECT COUNT(E.*)
            FROM event E
            WHERE E.block_number = $3
            AND E.metadata_event_id IN (SELECT id FROM matching_event_ids)
            "#,
        )
        .bind(format!("%{}%", query.pallet_name.as_deref().unwrap_or("")))
        .bind(format!("%{}%", query.event_name.as_deref().unwrap_or("")))
        .bind(block_number as i64)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_number(
        &self,
        page: u64,
        page_size: u64,
        block_number: u64,
        query: &BlockEventQuery,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            WITH matching_event_ids AS (
                SELECT ME.id, MP.name AS pallet_name, MP.index AS pallet_index, ME.index AS pallet_event_index, ME.name AS pallet_event_name
                FROM metadata_event ME
                INNER JOIN metadata_pallet MP ON ME.pallet_id = MP.id
                WHERE MP.name ILIKE $1
                AND ME.name ILIKE $2
            )
            SELECT E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status, E.trace_index, M.pallet_index, M.pallet_name, M.pallet_event_index, M.pallet_event_name, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args
            FROM event E
            INNER JOIN matching_event_ids M ON E.metadata_event_id = M.id
            WHERE block_number = $3
            ORDER BY E.index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
            .bind(format!("%{}%", query.pallet_name.as_deref().unwrap_or("")))
        .bind(format!("%{}%", query.event_name.as_deref().unwrap_or("")))
            .bind(block_number as i64)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(event_rows)
    }
}

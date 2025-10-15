use crate::types::persistence::EventCompositeRow;
use submerge_persistence::postgres::PostgreSQLStorage;

pub(crate) trait CrystalEventAPIPostgreSQLStorage {
    async fn get_event_count(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64>;
    async fn get_events(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u64,
        page_size: u64,
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
        page: u64,
        page_size: u64,
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
        page: u64,
        page_size: u64,
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
        page: u64,
        page_size: u64,
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
        page: u64,
        page_size: u64,
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
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<EventCompositeRow>>;
}

impl CrystalEventAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_event_count(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR E.block_number >= $1)
                AND ($2 IS NULL OR E.block_number <= $2)
                AND ($3 IS NULL OR E.block_timestamp >= $3)
                AND ($4 IS NULL OR E.block_timestamp <= $4)
                AND ($5 IS NULL OR E.spec_version >= $5)
                AND ($6 IS NULL OR E.spec_version <= $6)
                AND ($7 IS NULL OR MP.name ILIKE '%' || $7 || '%')
                AND ($8 IS NULL OR ME.name ILIKE '%' || $8 || '%')
            "#,
        )
        .bind(min_block_number.map(|n| n as i64))
        .bind(max_block_number.map(|n| n as i64))
        .bind(min_block_timestamp.map(|n| n as i64))
        .bind(max_block_timestamp.map(|n| n as i64))
        .bind(min_spec_version.map(|n| n as i32))
        .bind(max_spec_version.map(|n| n as i32))
        .bind(pallet_name)
        .bind(pallet_event_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_events(
        &self,
        min_block_number: Option<u64>,
        max_block_number: Option<u64>,
        min_block_timestamp: Option<u64>,
        max_block_timestamp: Option<u64>,
        min_spec_version: Option<u32>,
        max_spec_version: Option<u32>,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR E.block_number >= $1)
                AND ($2 IS NULL OR E.block_number <= $2)
                AND ($3 IS NULL OR E.block_timestamp >= $3)
                AND ($4 IS NULL OR E.block_timestamp <= $4)
                AND ($5 IS NULL OR E.spec_version >= $5)
                AND ($6 IS NULL OR E.spec_version <= $6)
                AND ($7 IS NULL OR MP.name ILIKE '%' || $7 || '%')
                AND ($8 IS NULL OR ME.name ILIKE '%' || $8 || '%')
            ORDER BY E.block_number DESC, E.index ASC
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
        .bind(pallet_event_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(event_rows)
    }

    async fn get_event_count_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR MP.name ILIKE '%' || $1 || '%')
                AND ($2 IS NULL OR ME.name ILIKE '%' || $2 || '%')
                AND E.block_hash = $3
            "#,
        )
        .bind(pallet_name)
        .bind(pallet_event_name)
        .bind(block_hash)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_hash(
        &self,
        block_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR MP.name ILIKE '%' || $1 || '%')
                AND ($2 IS NULL OR ME.name ILIKE '%' || $2 || '%')
                AND E.block_hash = $3
            ORDER BY E.index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(pallet_name)
        .bind(pallet_event_name)
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
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR MP.name ILIKE '%' || $1 || '%')
                AND ($2 IS NULL OR ME.name ILIKE '%' || $2 || '%')
                AND E.block_number = $3
            "#,
        )
        .bind(pallet_name)
        .bind(pallet_event_name)
        .bind(block_number as i64)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_events_by_block_number(
        &self,
        block_number: u64,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE
                ($1 IS NULL OR MP.name ILIKE '%' || $1 || '%')
                AND ($2 IS NULL OR ME.name ILIKE '%' || $2 || '%')
                AND E.block_number = $3
            ORDER BY E.index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(pallet_name)
        .bind(pallet_event_name)
        .bind(block_number as i64)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(event_rows)
    }

    async fn get_events_by_block_number_and_index(
        &self,
        block_number: u64,
        index: u32,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.block_number = $1 AND E.index = $2
            "#,
        )
        .bind(block_number as i64)
        .bind(index as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(event_rows)
    }

    async fn get_event_by_block_hash_and_index(
        &self,
        block_hash: &[u8],
        index: u32,
    ) -> anyhow::Result<Option<EventCompositeRow>> {
        let event_row: Option<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.block_hash = $1 AND E.index = $2
            "#,
        )
        .bind(block_hash)
        .bind(index as i64)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(event_row)
    }

    async fn get_event_count_by_block_number_and_extrinsic_index(
        &self,
        block_number: u64,
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.block_number = $1 AND E.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR ME.name ILIKE '%' || $4 || '%')
            "#,
        )
        .bind(block_number as i64)
        .bind(extrinsic_index as i64)
        .bind(pallet_name)
        .bind(pallet_event_name)
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
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.block_number = $1 AND E.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR ME.name ILIKE '%' || $4 || '%')
            ORDER BY E.index ASC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(block_number as i64)
        .bind(extrinsic_index as i64)
        .bind(pallet_name)
        .bind(pallet_event_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(event_rows)
    }

    async fn get_event_count_by_block_hash_and_extrinsic_index(
        &self,
        block_hash: &[u8],
        extrinsic_index: u32,
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.block_hash = $1 AND E.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR ME.name ILIKE '%' || $4 || '%')
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index as i64)
        .bind(pallet_name)
        .bind(pallet_event_name)
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
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.block_hash = $1 AND E.extrinsic_index = $2
                AND ($3 IS NULL OR MP.name ILIKE '%' || $3 || '%')
                AND ($4 IS NULL OR ME.name ILIKE '%' || $4 || '%')
            ORDER BY E.index ASC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index as i64)
        .bind(pallet_name)
        .bind(pallet_event_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(event_rows)
    }

    async fn get_event_count_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
    ) -> anyhow::Result<u64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.extrinsic_hash = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR ME.name ILIKE '%' || $3 || '%')
            "#,
        )
        .bind(extrinsic_hash)
        .bind(pallet_name)
        .bind(pallet_event_name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count as u64)
    }

    async fn get_events_by_extrinsic_hash(
        &self,
        extrinsic_hash: &[u8],
        pallet_name: &Option<String>,
        pallet_event_name: &Option<String>,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<EventCompositeRow>> {
        let offset = (page - 1) * page_size;
        let event_rows: Vec<EventCompositeRow> = sqlx::query_as(
            r#"
            SELECT
                E.id, E.block_hash, E.block_number, E.block_timestamp, E.spec_version, E.block_status,
                E.trace_index, E.extrinsic_index, E.extrinsic_hash, E.phase, E.index, E.args,
                MP.index AS pallet_index, MP.name AS pallet_name,
                ME.index AS pallet_event_index, ME.name AS pallet_event_name
            FROM event E
            JOIN metadata_event ME ON E.metadata_event_id = ME.id
            JOIN metadata_pallet MP ON ME.pallet_id = MP.id
            WHERE E.extrinsic_hash = $1
                AND ($2 IS NULL OR MP.name ILIKE '%' || $2 || '%')
                AND ($3 IS NULL OR ME.name ILIKE '%' || $3 || '%')
            ORDER BY E.index ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(extrinsic_hash)
        .bind(pallet_name)
        .bind(pallet_event_name)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(event_rows)
    }
}

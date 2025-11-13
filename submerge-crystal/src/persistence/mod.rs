use frame_metadata::RuntimeMetadataPrefixed;
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;
use serde_json::Value as JSONValue;
use sp_runtime::DigestItem;
use sqlx::QueryBuilder;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::block::DecodedBlockHeader;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_util::substrate::storage::get_storage_plain_key;

use crate::types::metadata::Metadata;
use crate::types::metadata::MetadataPallet;
use crate::types::persistence::{BlockRow, EventRow, ExtrinsicRow, LogRow};
use crate::types::BlockStatus;
use crate::types::Extrinsic;

pub mod api;

const INSERT_BATCH_SIZE: usize = 1000;

pub(crate) trait CrystalPostgreSQLStorage {
    async fn set_last_indexed_finalized_block_number_and_hash(
        &self,
        number: u64,
        hash: &[u8],
    ) -> anyhow::Result<()>;
    async fn get_last_indexed_finalized_block_number_and_hash(
        &self,
    ) -> anyhow::Result<Option<(u64, Vec<u8>)>>;
    async fn metadata_exists(&self, spec_version: u32) -> anyhow::Result<bool>;
    async fn get_metadata(
        &self,
        spec_version: u32,
    ) -> anyhow::Result<Option<RuntimeMetadataPrefixed>>;
    async fn ingest_metadata(
        &self,
        spec_version: u32,
        version: u32,
        metadata_bytes: &[u8],
        metadata_json: &JSONValue,
        metadata: &Metadata,
    ) -> anyhow::Result<()>;
    async fn ingest_metadata_pallet(
        &self,
        spec_version: u32,
        pallet: &MetadataPallet,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn get_metadata_pallet_id(&self, spec_version: u32, index: u8) -> anyhow::Result<u32>;
    async fn get_metadata_event_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32>;
    async fn get_metadata_call_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32>;
    async fn get_metadata_constant_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32>;
    async fn get_metadata_error_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32>;
    async fn get_metadata_storage_item_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32>;
    async fn get_genesis_record_count(&self) -> anyhow::Result<u64>;
    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()>;
    async fn get_next_block_number(
        &self,
        min: u64,
        max: u64,
        status: BlockStatus,
    ) -> anyhow::Result<u64>;
    #[cfg(test)]
    async fn get_error_count(&self) -> anyhow::Result<u32>;
    async fn save_error(
        &self,
        block_hash: &[u8],
        block_number: u64,
        status: BlockStatus,
        description: &str,
    ) -> anyhow::Result<()>;
    async fn delete_error(
        &self,
        block_hash: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn ingest_block(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        timestamp: Option<u64>,
        status: BlockStatus,
        weight: &Option<JSONValue>,
        spec_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        author_multi_address: &Option<MultiAddress>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn delete_block_and_traces_by_hash(
        &self,
        hash: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<bool>;
    async fn update_block_status(
        &self,
        block_hash: &[u8],
        status: BlockStatus,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn block_exists_by_hash(&self, hash: &[u8]) -> anyhow::Result<bool>;
    async fn get_block_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<BlockRow>>;
    async fn block_exists_by_number(&self, number: u64) -> anyhow::Result<bool>;
    async fn get_blocks_by_number(&self, number: u64) -> anyhow::Result<Vec<BlockRow>>;
    async fn get_blocks_by_number_with_tx(
        &self,
        number: u64,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Vec<BlockRow>>;
    async fn ingest_block_trace(
        &self,
        block_hash: &[u8],
        block_number: u64,
        spec_version: u32,
        trace_index: u32,
        key_prefix: &[u8],
        key_params: Option<&[u8]>,
        value: Option<&[u8]>,
        ext_id: &[u8],
        method: &str,
        parent_id: Option<&str>,
        metadata_storage_item_id: Option<u32>,
        is_known_key: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn ingest_block_logs(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn extrinsic_exists_by_hash(&self, hash: &[u8]) -> anyhow::Result<bool>;
    async fn ingest_extrinsics(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: Option<u64>,
        spec_version: u32,
        status: BlockStatus,
        extrinsics: &[Extrinsic],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Vec<(i64, i32)>>;
    async fn ingest_events(
        &self,
        event_rows: &[EventRow],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_call(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: Option<u64>,
        spec_version: u32,
        status: BlockStatus,
        extrinsic_id: i64,
        extrinsic_index: u32,
        extrinsic_hash: &[u8],
        parent_call_hash: Option<&[u8]>,
        call_path: &str,
        call_index: &[u16],
        metadata_call_id: u32,
        args: &JSONValue,
        extrinsic_is_successful: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Vec<u8>>;
    async fn ingest_genesis_item(
        key: &[u8],
        value: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO genesis (key, value)
            VALUES ($1, $2)
            ON CONFLICT(key) DO UPDATE SET
                value = EXCLUDED.value
                "#,
        )
        .bind(key)
        .bind(value)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

impl CrystalPostgreSQLStorage for PostgreSQLStorage {
    async fn set_last_indexed_finalized_block_number_and_hash(
        &self,
        number: u64,
        hash: &[u8],
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE crystal_state
            SET last_indexed_finalized_block_number = $1, last_indexed_finalized_block_hash = $2
            WHERE id = 1
            "#,
        )
        .bind(number as i64)
        .bind(hash)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
    async fn get_last_indexed_finalized_block_number_and_hash(
        &self,
    ) -> anyhow::Result<Option<(u64, Vec<u8>)>> {
        let row: (Option<i64>, Option<Vec<u8>>) = sqlx::query_as(
            r#"
            SELECT last_indexed_finalized_block_number, last_indexed_finalized_block_hash
            FROM crystal_state
            WHERE id = 1
            "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(match (row.0, row.1) {
            (Some(number), Some(hash)) => Some((number as u64, hash)),
            _ => None,
        })
    }

    async fn metadata_exists(&self, spec_version: u32) -> anyhow::Result<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(DISTINCT spec_version)
            FROM metadata
            WHERE spec_version = $1"#,
        )
        .bind(spec_version as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count > 0)
    }

    async fn get_metadata(
        &self,
        spec_version: u32,
    ) -> anyhow::Result<Option<RuntimeMetadataPrefixed>> {
        let maybe_metadata_bytes: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT metadata_bytes FROM metadata WHERE spec_version = $1")
                .bind(spec_version as i32)
                .fetch_optional(&self.connection_pool)
                .await?;
        if let Some(metadata_bytes) = maybe_metadata_bytes {
            let mut metadata_bytes: &[u8] = metadata_bytes.0.as_ref();
            let metadata = RuntimeMetadataPrefixed::decode(&mut metadata_bytes)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    async fn ingest_metadata(
        &self,
        spec_version: u32,
        metadata_version: u32,
        metadata_bytes: &[u8],
        metadata_json: &JSONValue,
        metadata: &Metadata,
    ) -> anyhow::Result<()> {
        if self.metadata_exists(spec_version).await? {
            return Ok(());
        }
        let mut tx = self.connection_pool.begin().await?;
        if sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO metadata (spec_version, metadata_version, metadata_bytes, metadata_json)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (spec_version) DO UPDATE SET
                metadata_version = EXCLUDED.metadata_version, metadata_bytes = EXCLUDED.metadata_bytes,
                metadata_json = EXCLUDED.metadata_json
            RETURNING spec_version
            "#,
        )
        .bind(spec_version as i32)
        .bind(metadata_version as i32)
        .bind(metadata_bytes)
        .bind(metadata_json)
        .fetch_optional(&mut *tx)
        .await?
        .is_some()
        {
            for pallet in metadata.pallets.iter() {
                self.ingest_metadata_pallet(spec_version, pallet, &mut tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    async fn ingest_metadata_pallet(
        &self,
        spec_version: u32,
        pallet: &MetadataPallet,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let pallet_id_row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO metadata_pallet (spec_version, index, name)
            VALUES ($1, $2, $3)
            ON CONFLICT (spec_version, index) DO UPDATE SET
                name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet.index as i32)
        .bind(&pallet.name)
        .fetch_one(&mut **tx)
        .await?;
        let pallet_id = pallet_id_row.0;
        for event in pallet.events.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_event (pallet_id, index, name, docs)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (pallet_id, index) DO UPDATE SET
                    name = EXCLUDED.name, docs = EXCLUDED.docs
                "#,
            )
            .bind(pallet_id)
            .bind(event.index as i32)
            .bind(&event.name)
            .bind(&event.docs)
            .execute(&mut **tx)
            .await?;
        }
        for constant in pallet.constants.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_constant (pallet_id, index, name, type_id, type_name, value, value_json, docs)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (pallet_id, index) DO UPDATE SET
                    name = EXCLUDED.name, type_id = EXCLUDED.type_id, type_name = EXCLUDED.type_name, 
                    value = EXCLUDED.value, value_json = EXCLUDED.value_json, docs = EXCLUDED.docs
                "#,
            )
            .bind(pallet_id)
            .bind(constant.index as i32)
            .bind(&constant.name)
            .bind(constant.type_id.map(|i| i as i32))
            .bind(&constant.type_name)
            .bind(&constant.value)
            .bind(&constant.value_json)
            .bind(&constant.docs)
            .execute(&mut **tx)
            .await?;
        }
        for call in pallet.calls.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_call (pallet_id, index, name, docs)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (pallet_id, index) DO UPDATE SET
                    name = EXCLUDED.name, docs = EXCLUDED.docs
                "#,
            )
            .bind(pallet_id)
            .bind(call.index as i32)
            .bind(&call.name)
            .bind(&call.docs)
            .execute(&mut **tx)
            .await?;
        }
        for storage_item in pallet.storage_items.iter() {
            let key = get_storage_plain_key(&pallet.name, &storage_item.name);
            sqlx::query(
                r#"
                INSERT INTO metadata_storage_item (pallet_id, index, name, key_prefix, docs)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (pallet_id, index) DO UPDATE SET
                    name = EXCLUDED.name, key_prefix = EXCLUDED.key_prefix, docs = EXCLUDED.docs
                "#,
            )
            .bind(pallet_id)
            .bind(storage_item.index as i32)
            .bind(&storage_item.name)
            .bind(&key)
            .bind(&storage_item.docs)
            .execute(&mut **tx)
            .await?;
        }
        for error in pallet.errors.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_error (pallet_id, index, name, docs)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (pallet_id, index) DO UPDATE SET
                    name = EXCLUDED.name, docs = EXCLUDED.docs
                "#,
            )
            .bind(pallet_id)
            .bind(error.index as i32)
            .bind(&error.name)
            .bind(&error.docs)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    async fn get_metadata_pallet_id(&self, spec_version: u32, index: u8) -> anyhow::Result<u32> {
        let id_row: (i32,) =
            sqlx::query_as("SELECT id FROM metadata_pallet WHERE spec_version = $1 AND index = $2")
                .bind(spec_version as i32)
                .bind(index as i32)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(id_row.0 as u32)
    }

    async fn get_metadata_event_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32> {
        let id_row: (i32,) =
            sqlx::query_as("SELECT id FROM metadata_event WHERE pallet_id = $1 AND index = $2")
                .bind(pallet_id as i32)
                .bind(index as i32)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(id_row.0 as u32)
    }

    async fn get_metadata_call_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32> {
        let id_row: (i32,) =
            sqlx::query_as("SELECT id FROM metadata_call WHERE pallet_id = $1 AND index = $2")
                .bind(pallet_id as i32)
                .bind(index as i32)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(id_row.0 as u32)
    }

    async fn get_metadata_constant_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32> {
        let id_row: (i32,) =
            sqlx::query_as("SELECT id FROM metadata_constant WHERE pallet_id = $1 AND index = $2")
                .bind(pallet_id as i32)
                .bind(index as i32)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(id_row.0 as u32)
    }

    async fn get_metadata_error_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32> {
        let id_row: (i32,) =
            sqlx::query_as("SELECT id FROM metadata_error WHERE pallet_id = $1 AND index = $2")
                .bind(pallet_id as i32)
                .bind(index as i32)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(id_row.0 as u32)
    }

    async fn get_metadata_storage_item_id(&self, pallet_id: u32, index: u8) -> anyhow::Result<u32> {
        let id_row: (i32,) = sqlx::query_as(
            "SELECT id FROM metadata_storage_item WHERE pallet_id = $1 AND index = $2",
        )
        .bind(pallet_id as i32)
        .bind(index as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(id_row.0 as u32)
    }

    async fn get_genesis_record_count(&self) -> anyhow::Result<u64> {
        let record_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM genesis")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(record_count.0 as u64)
    }

    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()> {
        let mut tx = self.connection_pool.begin().await?;
        for (key, value) in chainspec.genesis.raw.top.iter() {
            Self::ingest_genesis_item(
                &hex::decode(key.trim_start_matches("0x"))?,
                &hex::decode(value.trim_start_matches("0x"))?,
                &mut tx,
            )
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn get_next_block_number(
        &self,
        min: u64,
        max: u64,
        status: BlockStatus,
    ) -> anyhow::Result<u64> {
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT MAX(number) FROM block WHERE number >= $1 AND number <= $2 AND status = $3",
        )
        .bind(min as i64)
        .bind(max as i64)
        .bind(status)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(if let Some(min_in_range) = row.0 {
            min_in_range as u64 + 1
        } else {
            min
        })
    }

    #[cfg(test)]
    async fn get_error_count(&self) -> anyhow::Result<u32> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT block_number) FROM error")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(row.0 as u32)
    }

    async fn save_error(
        &self,
        block_hash: &[u8],
        block_number: u64,
        status: BlockStatus,
        description: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO error (block_hash, block_number, block_status, description)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(block_hash) DO UPDATE SET
                block_number = EXCLUDED.block_number, block_status = EXCLUDED.block_status,
                description = EXCLUDED.description, created_at = now()
            "#,
        )
        .bind(block_hash)
        .bind(block_number as i64)
        .bind(status)
        .bind(description)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    async fn delete_error(
        &self,
        block_hash: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM error WHERE block_hash = $1")
            .bind(block_hash)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn delete_block_and_traces_by_hash(
        &self,
        hash: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<bool> {
        let block_delete_result = sqlx::query("DELETE FROM block WHERE hash = $1")
            .bind(hash)
            .execute(&mut **tx)
            .await?;
        sqlx::query("DELETE FROM trace WHERE block_hash = $1")
            .bind(hash)
            .execute(&mut **tx)
            .await?;
        Ok(block_delete_result.rows_affected() == 1)
    }

    async fn ingest_block(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        timestamp: Option<u64>,
        status: BlockStatus,
        weight: &Option<JSONValue>,
        spec_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        author_multi_address: &Option<MultiAddress>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let header = DecodedBlockHeader::try_from(header)?;
        sqlx::query(
            r#"
                INSERT INTO block (hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_multi_address)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (hash) DO UPDATE SET
                    parent_hash = EXCLUDED.parent_hash, state_root = EXCLUDED.state_root, extrinsic_root = EXCLUDED.extrinsic_root,
                    number = EXCLUDED.number, timestamp = EXCLUDED.timestamp, spec_version = EXCLUDED.spec_version, status = EXCLUDED.status,
                    weight = EXCLUDED.weight, extrinsic_count = EXCLUDED.extrinsic_count, event_count = EXCLUDED.event_count,
                    author_multi_address = EXCLUDED.author_multi_address
                "#,
        )
            .bind(hash)
            .bind(&header.parent_hash)
            .bind(&header.state_root)
            .bind(&header.extrinsic_root)
            .bind(header.number as i64)
            .bind(timestamp.map(|t| t as i64))
            .bind(spec_version as i32)
            .bind(status)
            .bind(weight)
            .bind(extrinsic_count as i32)
            .bind(event_count as i32)
            .bind(author_multi_address.as_ref().map(|multi_address| multi_address.encode()))
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn update_block_status(
        &self,
        block_hash: &[u8],
        status: BlockStatus,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE block SET status = $1 WHERE hash = $2")
            .bind(status)
            .bind(block_hash)
            .execute(&mut **tx)
            .await?;
        sqlx::query("UPDATE extrinsic SET block_status = $1 WHERE block_hash = $2")
            .bind(status)
            .bind(block_hash)
            .execute(&mut **tx)
            .await?;
        sqlx::query("UPDATE call SET block_status = $1 WHERE block_hash = $2")
            .bind(status)
            .bind(block_hash)
            .execute(&mut **tx)
            .await?;
        sqlx::query("UPDATE event SET block_status = $1 WHERE block_hash = $2")
            .bind(status)
            .bind(block_hash)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn block_exists_by_hash(&self, hash: &[u8]) -> anyhow::Result<bool> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM block WHERE hash = $1)")
            .bind(hash)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(exists)
    }

    async fn get_block_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<BlockRow>> {
        let maybe_row: Option<BlockRow> = sqlx::query_as(
            r#"
            SELECT hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_multi_address
            FROM block
            WHERE hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_row)
    }

    async fn block_exists_by_number(&self, number: u64) -> anyhow::Result<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM block WHERE number = $1)")
                .bind(number as i64)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(exists)
    }

    async fn get_blocks_by_number(&self, number: u64) -> anyhow::Result<Vec<BlockRow>> {
        let rows: Vec<BlockRow> = sqlx::query_as(
            r#"
            SELECT hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_multi_address
            FROM block
            WHERE number = $1
            "#,
        )
        .bind(number as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    async fn get_blocks_by_number_with_tx(
        &self,
        number: u64,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Vec<BlockRow>> {
        let rows: Vec<BlockRow> = sqlx::query_as(
            r#"
            SELECT hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_multi_address
            FROM block
            WHERE number = $1
            "#,
        )
        .bind(number as i64)
        .fetch_all(&mut **tx)
        .await?;
        Ok(rows)
    }

    async fn ingest_block_trace(
        &self,
        block_hash: &[u8],
        block_number: u64,
        spec_version: u32,
        trace_index: u32,
        key_prefix: &[u8],
        key_params: Option<&[u8]>,
        value: Option<&[u8]>,
        ext_id: &[u8],
        method: &str,
        parent_id: Option<&str>,
        metadata_storage_item_id: Option<u32>,
        is_known_key: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO trace (block_hash, block_number, spec_version, index, key_prefix, key_params, value, ext_id, method, parent_id, metadata_storage_item_id, is_known_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (block_hash, block_number, index) DO UPDATE SET
                spec_version = EXCLUDED.spec_version, key_prefix = EXCLUDED.key_prefix, key_params = EXCLUDED.key_params,
                value = EXCLUDED.value, ext_id = EXCLUDED.ext_id, method = EXCLUDED.method, parent_id = EXCLUDED.parent_id,
                metadata_storage_item_id = EXCLUDED.metadata_storage_item_id, is_known_key = EXCLUDED.is_known_key
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(spec_version as i32)
            .bind(trace_index as i32)
            .bind(key_prefix)
            .bind(key_params)
            .bind(value)
            .bind(ext_id)
            .bind(method)
            .bind(parent_id)
            .bind(metadata_storage_item_id.map(|id| id as i32))
            .bind(is_known_key)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn ingest_block_logs(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let mut rows = Vec::new();
        for (index, log) in header.get_logs()?.iter().enumerate() {
            let (ty, engine, data) = match log {
                DigestItem::PreRuntime(engine, data) => {
                    let engine = String::from_utf8(engine.to_vec())?;
                    ("PreRuntime", Some(engine), Some(data))
                }
                DigestItem::Consensus(engine, data) => {
                    let engine = String::from_utf8(engine.to_vec())?;
                    ("Consensus", Some(engine), Some(data))
                }
                DigestItem::Seal(engine, data) => {
                    let engine = String::from_utf8(engine.to_vec())?;
                    ("Seal", Some(engine), Some(data))
                }
                DigestItem::Other(data) => ("Other", None, Some(data)),
                DigestItem::RuntimeEnvironmentUpdated => ("RuntimeEnvironmentUpdated", None, None),
            };
            rows.push(LogRow {
                block_hash: hash.into(),
                block_number: header.get_number()? as i64,
                index: index as i32,
                ty: ty.to_string(),
                engine,
                data: data.cloned(),
            });
        }
        for row_chunk in rows.chunks(INSERT_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO log (block_hash, block_number, index, type, engine, data) ",
            );
            query_builder.push_values(row_chunk, |mut query, log| {
                query
                    .push_bind(&log.block_hash)
                    .push_bind(log.block_number)
                    .push_bind(log.index)
                    .push_bind(&log.ty)
                    .push_bind(&log.engine)
                    .push_bind(&log.data);
            });
            query_builder.push(
                r#"
                ON CONFLICT (block_hash, index) DO UPDATE SET 
                    block_number = EXCLUDED.block_number, type = EXCLUDED.type,
                    engine = EXCLUDED.engine, data = EXCLUDED.data
                "#,
            );
            let query: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> =
                query_builder.build();
            query.execute(&mut **tx).await?;
        }
        Ok(())
    }

    async fn extrinsic_exists_by_hash(&self, hash: &[u8]) -> anyhow::Result<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM extrinsic WHERE hash = $1)")
                .bind(hash)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(exists)
    }

    async fn ingest_extrinsics(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: Option<u64>,
        spec_version: u32,
        block_status: BlockStatus,
        extrinsics: &[Extrinsic],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Vec<(i64, i32)>> {
        let extrinsic_rows: Vec<ExtrinsicRow> = extrinsics
            .iter()
            .map(|extrinsic| {
                let (signer, signature, extra) = if let Some(signature) = &extrinsic.signature {
                    (
                        Some(Encode::encode(&signature.signer)),
                        Some(Encode::encode(&signature.signature)),
                        signature.extra.clone(),
                    )
                } else {
                    (None, None, None)
                };
                ExtrinsicRow {
                    id: 0,
                    block_hash: block_hash.into(),
                    block_number: block_number as i64,
                    block_timestamp: block_timestamp.map(|timestamp| timestamp as i64),
                    spec_version: spec_version as i32,
                    block_status,
                    trace_index: extrinsic.trace_index.map(|i| i as i32),
                    hash: extrinsic.hash,
                    index: extrinsic.index as i32,
                    version: extrinsic.version as i32,
                    signer_multi_address: signer,
                    multi_signature: signature,
                    extra,
                    is_successful: extrinsic.is_successful,
                }
            })
            .collect();
        let mut ids_to_indices = Vec::new();
        for extrinsic_row_chunk in extrinsic_rows.chunks(INSERT_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO extrinsic (block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer_multi_address, multi_signature, extra, is_successful) ",
            );
            query_builder.push_values(extrinsic_row_chunk, |mut query, extrinsic| {
                query
                    .push_bind(&extrinsic.block_hash)
                    .push_bind(extrinsic.block_number)
                    .push_bind(extrinsic.block_timestamp)
                    .push_bind(extrinsic.spec_version)
                    .push_bind(extrinsic.block_status)
                    .push_bind(extrinsic.trace_index)
                    .push_bind(extrinsic.hash)
                    .push_bind(extrinsic.index)
                    .push_bind(extrinsic.version)
                    .push_bind(&extrinsic.signer_multi_address)
                    .push_bind(&extrinsic.multi_signature)
                    .push_bind(&extrinsic.extra)
                    .push_bind(extrinsic.is_successful);
            });
            query_builder.push(
                r#"
                ON CONFLICT (block_hash, block_number, index) DO UPDATE SET 
                    block_timestamp = EXCLUDED.block_timestamp, spec_version = EXCLUDED.spec_version,
                    block_status = EXCLUDED.block_status, trace_index = EXCLUDED.trace_index, hash = EXCLUDED.hash,
                    index = EXCLUDED.index, version = EXCLUDED.version, signer_multi_address = EXCLUDED.signer_multi_address,
                    multi_signature = EXCLUDED.multi_signature, extra = EXCLUDED.extra, is_successful = EXCLUDED.is_successful
                RETURNING id, index
                "#
            );
            let rows: Vec<(i64, i32)> = query_builder.build_query_as().fetch_all(&mut **tx).await?;
            ids_to_indices.extend(rows);
        }
        Ok(ids_to_indices)
    }

    async fn ingest_events(
        &self,
        event_rows: &[EventRow],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        for event_row_chunk in event_rows.chunks(INSERT_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO event (block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, metadata_event_id, extrinsic_index, extrinsic_hash, phase, index, args) ",
            );
            query_builder.push_values(event_row_chunk, |mut query, event| {
                query
                    .push_bind(&event.block_hash)
                    .push_bind(event.block_number)
                    .push_bind(event.block_timestamp)
                    .push_bind(event.spec_version)
                    .push_bind(event.block_status)
                    .push_bind(event.trace_index)
                    .push_bind(event.metadata_event_id)
                    .push_bind(event.extrinsic_index)
                    .push_bind(event.extrinsic_hash)
                    .push_bind(&event.phase)
                    .push_bind(event.index)
                    .push_bind(&event.args);
            });
            query_builder.push(
                r#"
                ON CONFLICT (block_hash, block_number, index) DO UPDATE SET
                    block_timestamp = EXCLUDED.block_timestamp, spec_version = EXCLUDED.spec_version, block_status = EXCLUDED.block_status,
                    trace_index = EXCLUDED.trace_index, metadata_event_id = EXCLUDED.metadata_event_id,
                    extrinsic_index = EXCLUDED.extrinsic_index, extrinsic_hash = EXCLUDED.extrinsic_hash, phase = EXCLUDED.phase,
                    index = EXCLUDED.index, args = EXCLUDED.args
                "#);
            let query: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> =
                query_builder.build();
            query.execute(&mut **tx).await?;
        }
        Ok(())
    }

    async fn ingest_call(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: Option<u64>,
        spec_version: u32,
        block_status: BlockStatus,
        extrinsic_id: i64,
        extrinsic_index: u32,
        extrinsic_hash: &[u8],
        parent_call_hash: Option<&[u8]>,
        call_path: &str,
        call_index: &[u16],
        metadata_call_id: u32,
        args: &JSONValue,
        extrinsic_is_successful: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Vec<u8>> {
        let call_index: Vec<i16> = call_index.iter().map(|&x| x as i16).collect();
        let row: (Vec<u8>,) = sqlx::query_as(
            r#"
            INSERT INTO call (block_hash, block_number, block_timestamp, spec_version, block_status, extrinsic_id, extrinsic_index, extrinsic_hash, parent_call_hash, call_path, call_index, metadata_call_id, extrinsic_is_successful, args)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (block_number, hash) DO UPDATE SET
                block_timestamp = EXCLUDED.block_timestamp, spec_version = EXCLUDED.spec_version, block_status = EXCLUDED.block_status,
                extrinsic_id = EXCLUDED.extrinsic_id, extrinsic_index = EXCLUDED.extrinsic_index, extrinsic_hash = EXCLUDED.extrinsic_hash,
                parent_call_hash = EXCLUDED.parent_call_hash, call_path = EXCLUDED.call_path, call_index = EXCLUDED.call_index,
                metadata_call_id = EXCLUDED.metadata_call_id, extrinsic_is_successful = EXCLUDED.extrinsic_is_successful, args = EXCLUDED.args
            RETURNING hash
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp.map(|timestamp| timestamp as i64))
            .bind(spec_version as i32)
            .bind(block_status)
            .bind(extrinsic_id)
            .bind(extrinsic_index as i32)
            .bind(extrinsic_hash)
            .bind(parent_call_hash)
            .bind(call_path)
            .bind(call_index)
            .bind(metadata_call_id as i32)
            .bind(extrinsic_is_successful)
            .bind(args)
            .fetch_one(&mut **tx)
            .await?;
        Ok(row.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        persistence::{CrystalPostgreSQLStorage, PostgreSQLStorage},
        types::BlockStatus,
    };
    use anyhow::Context as _;
    use std::{cmp::min, fs};
    use submerge_base::{args::PostgreSQLArgs, types::substrate::chainspec::Chainspec};
    use submerge_substrate_client::{RPCConfig, SubstrateClient};

    async fn get_test_postgres() -> anyhow::Result<PostgreSQLStorage> {
        let args = PostgreSQLArgs {
            postgres_host: "localhost".to_owned(),
            postgres_port: 5432,
            postgres_username: "submerge".to_owned(),
            postgres_password: "submerge".to_owned(),
            postgres_db_name: "submerge_crystal_test".to_owned(),
            postgres_connection_timeout_secs: 10,
            postgres_pool_max_connections: 100,
        };
        PostgreSQLStorage::new(&args).await
    }

    #[tokio::test]
    async fn test_genesis_ingestion() -> Result<(), Box<dyn std::error::Error>> {
        let chainspec_path = "../_chainspecs/westend/sys/coretime-westend.json";
        let chainspec_json = fs::read_to_string(chainspec_path)?;
        let chainspec: Chainspec = serde_json::from_str(&chainspec_json)?;
        let postgres = get_test_postgres().await?;
        postgres.ingest_genesis(&chainspec).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ingest_blocks() -> Result<(), Box<dyn std::error::Error>> {
        let postgres = get_test_postgres().await?;
        let rpc_config = RPCConfig {
            rpc_url: "wss://rpc.helikon.io/coretime-westend-dev".to_owned(),
            rpc_connection_timeout_secs: 30,
            rpc_request_timeout_secs: 30,
            rpc_subscription_timeout_secs: 30,
        };
        let substrate_client = SubstrateClient::new(&rpc_config).await?;
        for number in 100..150 {
            let Some(block_hash) = substrate_client.get_block_hash(number).await? else {
                return Err(anyhow::anyhow!("Block {number} not found on the RPC node.").into());
            };
            let header = substrate_client.get_block_header(&block_hash).await?;
            let block_number = header.get_number()?;
            let timestamp = substrate_client.get_block_timestamp(&block_hash).await?;
            let last_runtime_upgrade = substrate_client
                .get_last_runtime_upgrade_info(&block_hash)
                .await?;
            let trace = substrate_client.get_block_trace(&block_hash).await?;
            let block_hash = hex::decode(block_hash)?;
            let mut tx = postgres.connection_pool.begin().await?;
            postgres
                .ingest_block(
                    &block_hash,
                    &header,
                    timestamp,
                    BlockStatus::Proposed,
                    &None,
                    last_runtime_upgrade.spec_version,
                    0,
                    0,
                    &None,
                    &mut tx,
                )
                .await?;
            postgres
                .ingest_block_logs(&block_hash, &header, &mut tx)
                .await?;
            for (trace_index, event) in trace.events.iter().enumerate() {
                let key = hex::decode(event.data_wrapper.data.key.trim_start_matches("0x"))
                    .context(format!(
                        "Cannot decode key for trace #{} in block #{}.",
                        trace_index, block_number,
                    ))?;
                let ext_id = hex::decode(event.data_wrapper.data.ext_id.trim_start_matches("0x"))
                    .context(format!(
                    "Cannot decode ext id for trace #{} in block #{}.",
                    trace_index, block_number,
                ))?;
                let value = if event.data_wrapper.data.value.is_empty()
                    || event.data_wrapper.data.value.eq_ignore_ascii_case("none")
                {
                    None
                } else if let Some(inner) = event
                    .data_wrapper
                    .data
                    .value
                    .to_lowercase()
                    .strip_prefix("some(")
                    .and_then(|s| s.strip_suffix(')'))
                {
                    Some(hex::decode(inner).context("Cannot decode trace value hex string.")?)
                } else {
                    Some(
                        hex::decode(&event.data_wrapper.data.value)
                            .context("Cannot decode trace value hex string.")?,
                    )
                };
                let key_prefix = &key[..min(key.len(), 32)];
                let key_params = key.get(32..);
                postgres
                    .ingest_block_trace(
                        &block_hash,
                        block_number,
                        last_runtime_upgrade.spec_version,
                        trace_index as u32,
                        key_prefix,
                        key_params,
                        value.as_deref(),
                        &ext_id,
                        &event.data_wrapper.data.method.to_string(),
                        event.parent_id.as_deref(),
                        None,
                        false,
                        &mut tx,
                    )
                    .await?;
            }
            postgres.delete_error(&block_hash, &mut tx).await?;
            tx.commit().await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_trace_error() -> Result<(), Box<dyn std::error::Error>> {
        let postgres = get_test_postgres().await?;
        let block_number = 100;
        let rpc_config = RPCConfig {
            rpc_url: "wss://rpc.helikon.io/coretime-westend-dev".to_owned(),
            rpc_connection_timeout_secs: 30,
            rpc_request_timeout_secs: 30,
            rpc_subscription_timeout_secs: 30,
        };
        let substrate_client = SubstrateClient::new(&rpc_config).await?;
        let Some(block_hash) = substrate_client.get_block_hash(block_number).await? else {
            return Err(anyhow::anyhow!("Block {block_number} not found on the RPC node.").into());
        };
        let block_hash = hex::decode(block_hash)?;
        let mut tx = postgres.connection_pool.begin().await?;
        postgres.delete_error(&block_hash, &mut tx).await?;
        tx.commit().await?;
        let pre_trace_error_count = postgres.get_error_count().await?;
        postgres
            .save_error(
                &block_hash,
                block_number,
                BlockStatus::Finalized,
                "error_description",
            )
            .await?;
        let post_trace_error_count = postgres.get_error_count().await?;
        assert_eq!(post_trace_error_count, pre_trace_error_count + 1);
        Ok(())
    }
}

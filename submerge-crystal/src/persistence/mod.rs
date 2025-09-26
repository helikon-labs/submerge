use frame_metadata::RuntimeMetadataPrefixed;
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;
use serde_json::Value as JSONValue;
use sp_runtime::AccountId32;
use sp_runtime::DigestItem;
use sqlx::QueryBuilder;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::block::DecodedBlockHeader;
use submerge_base::types::substrate::block_trace::BlockTrace as SubstrateBlockTrace;
use submerge_base::types::substrate::chainspec::Chainspec;
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
    async fn get_pallet_index_by_name(
        &self,
        spec_version: u32,
        name: &str,
    ) -> anyhow::Result<Option<u8>>;
    async fn get_pallet_call_index_by_name(
        &self,
        spec_version: u32,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>>;
    async fn get_pallet_event_index_by_name(
        &self,
        spec_version: u32,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>>;
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
        timestamp: u64,
        status: BlockStatus,
        weight: &Option<JSONValue>,
        spec_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        author_account_id: &AccountId32,
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
        hash: &[u8],
        header: &BlockHeader,
        status: BlockStatus,
        spec_version: u32,
        trace: &SubstrateBlockTrace,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn ingest_block_logs(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        status: BlockStatus,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn ingest_extrinsics(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
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
        block_timestamp: u64,
        spec_version: u32,
        status: BlockStatus,
        extrinsic_id: i64,
        extrinsic_index: u32,
        extrinsic_hash: &[u8],
        parent_call_id: Option<i64>,
        nesting_index: Option<&str>,
        pallet_index: u8,
        pallet_name: &str,
        pallet_call_index: u8,
        pallet_call_name: &str,
        is_successful: bool,
        args: &JSONValue,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64>;

    async fn ingest_genesis_item(
        key: &str,
        value: &str,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO genesis (key, value) VALUES ($1, $2) ON CONFLICT(key) DO NOTHING")
            .bind(key)
            .bind(value)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

impl CrystalPostgreSQLStorage for PostgreSQLStorage {
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
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT spec_version)
            FROM metadata
            WHERE spec_version = $1
            "#,
        )
        .bind(spec_version as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        if record_count.0 > 0 {
            return Ok(());
        }

        let mut tx = self.connection_pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO metadata (spec_version, metadata_version, metadata_bytes, metadata_json)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(spec_version as i32)
        .bind(metadata_version as i32)
        .bind(metadata_bytes)
        .bind(metadata_json)
        .execute(&mut *tx)
        .await?;
        for pallet in metadata.pallets.iter() {
            self.ingest_metadata_pallet(spec_version, pallet, &mut tx)
                .await?;
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
        sqlx::query(
            r#"
            INSERT INTO metadata_pallet (spec_version, index, name)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet.index as i32)
        .bind(&pallet.name)
        .execute(&mut **tx)
        .await?;
        for event in pallet.events.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_pallet_event (spec_version, pallet_index, pallet_name, index, name, docs)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
            .bind(event.index as i32)
            .bind(&event.name)
            .bind(&event.docs)
            .execute(&mut **tx)
            .await?;
        }
        for constant in pallet.constants.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_pallet_constant (spec_version, pallet_index, pallet_name, index, name, type_id, type_name, value, value_json, docs)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
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
                INSERT INTO metadata_pallet_call (spec_version, pallet_index, pallet_name, index, name, docs)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
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
                INSERT INTO metadata_pallet_storage_item (spec_version, pallet_index, pallet_name, index, name, key, docs)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
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
                INSERT INTO metadata_pallet_error (spec_version, pallet_index, pallet_name, index, name, docs)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
            .bind(error.index as i32)
            .bind(&error.name)
            .bind(&error.docs)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    async fn get_pallet_index_by_name(
        &self,
        spec_version: u32,
        name: &str,
    ) -> anyhow::Result<Option<u8>> {
        let maybe_row: Option<(i32,)> = sqlx::query_as(
            "SELECT index FROM metadata_pallet WHERE spec_version = $1 AND name = $2",
        )
        .bind(spec_version as i32)
        .bind(name)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_row.map(|row| row.0 as u8))
    }

    async fn get_pallet_call_index_by_name(
        &self,
        spec_version: u32,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>> {
        let maybe_row: Option<(i32,)> = sqlx::query_as(
            "SELECT index FROM metadata_pallet_call WHERE spec_version = $1 AND pallet_index = $2 AND name = $3",
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .bind(name)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_row.map(|row| row.0 as u8))
    }

    async fn get_pallet_event_index_by_name(
        &self,
        spec_version: u32,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>> {
        let maybe_row: Option<(i32,)> = sqlx::query_as(
            "SELECT index FROM metadata_pallet_event WHERE spec_version = $1 AND pallet_index = $2 AND name = $3",
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .bind(name)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_row.map(|row| row.0 as u8))
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
                key.trim_start_matches("0x"),
                value.trim_start_matches("0x"),
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
            ON CONFLICT(block_hash) DO UPDATE
            SET block_status = EXCLUDED.block_status, description = EXCLUDED.description, created_at = now()
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
        timestamp: u64,
        status: BlockStatus,
        weight: &Option<JSONValue>,
        spec_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        author_account_id: &AccountId32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let header = DecodedBlockHeader::try_from(header)?;
        let author_account_id: &[u8; 32] = author_account_id.as_ref();
        sqlx::query(
            r#"
                INSERT INTO block (hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_account_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (hash) DO NOTHING
                "#,
        )
            .bind(hash)
            .bind(&header.parent_hash)
            .bind(&header.state_root)
            .bind(&header.extrinsic_root)
            .bind(header.number as i64)
            .bind(timestamp as i64)
            .bind(spec_version as i32)
            .bind(status)
            .bind(weight)
            .bind(extrinsic_count as i32)
            .bind(event_count as i32)
            .bind(author_account_id)
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
        sqlx::query("UPDATE trace SET block_status = $1 WHERE block_hash = $2")
            .bind(status)
            .bind(block_hash)
            .execute(&mut **tx)
            .await?;
        sqlx::query("UPDATE log SET block_status = $1 WHERE block_hash = $2")
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
        let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM block WHERE hash = $1)")
            .bind(hash)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(exists.0)
    }

    async fn get_block_by_hash(&self, hash: &[u8]) -> anyhow::Result<Option<BlockRow>> {
        let maybe_row: Option<BlockRow> = sqlx::query_as(
            r#"
            SELECT hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_account_id
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
        let exists: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM block WHERE number = $1)")
                .bind(number as i64)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(exists.0)
    }

    async fn get_blocks_by_number(&self, number: u64) -> anyhow::Result<Vec<BlockRow>> {
        let rows: Vec<BlockRow> = sqlx::query_as(
            r#"
            SELECT hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_account_id
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
            SELECT hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, status, weight, extrinsic_count, event_count, author_account_id
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
        hash: &[u8],
        header: &BlockHeader,
        block_status: BlockStatus,
        spec_version: u32,
        trace: &SubstrateBlockTrace,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let header = DecodedBlockHeader::try_from(header)?;
        for (trace_index, event) in trace.events.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO trace (block_hash, block_parent_hash, block_number, spec_version, block_status, index, key, value, ext_id, method, parent_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (block_hash, block_number, index) DO NOTHING
                "#,
            )
                .bind(hash)
                .bind(&header.parent_hash)
                .bind(header.number as i64)
                .bind(spec_version as i32)
                .bind(block_status)
                .bind(trace_index as i32)
                .bind(&event.data_wrapper.data.key)
                .bind(&event.data_wrapper.data.value)
                .bind(&event.data_wrapper.data.ext_id)
                .bind(event.data_wrapper.data.method.to_string())
                .bind(&event.parent_id)
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }

    async fn ingest_block_logs(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        block_status: BlockStatus,
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
                block_status,
                index: index as i32,
                ty: ty.to_string(),
                engine,
                data: data.cloned(),
            });
        }
        for row_chunk in rows.chunks(INSERT_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO log (block_hash, block_number, block_status, index, type, engine, data) ",
            );
            query_builder.push_values(row_chunk, |mut query, log| {
                query
                    .push_bind(&log.block_hash)
                    .push_bind(log.block_number)
                    .push_bind(log.block_status)
                    .push_bind(log.index)
                    .push_bind(&log.ty)
                    .push_bind(&log.engine)
                    .push_bind(&log.data);
            });
            let query: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> =
                query_builder.build();
            query.execute(&mut **tx).await?;
        }
        Ok(())
    }

    async fn ingest_extrinsics(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
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
                    block_timestamp: block_timestamp as i64,
                    spec_version: spec_version as i32,
                    block_status,
                    trace_index: extrinsic.trace_index.map(|i| i as i32),
                    hash: extrinsic.hash,
                    index: extrinsic.index as i32,
                    version: extrinsic.version as i32,
                    signer,
                    signature,
                    extra,
                    is_successful: extrinsic.is_successful,
                }
            })
            .collect();
        let mut ids_to_indices = Vec::new();
        for extrinsic_row_chunk in extrinsic_rows.chunks(INSERT_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO extrinsic (block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, hash, index, version, signer, signature, extra, is_successful) ",
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
                    .push_bind(&extrinsic.signer)
                    .push_bind(&extrinsic.signature)
                    .push_bind(&extrinsic.extra)
                    .push_bind(extrinsic.is_successful);
            });
            query_builder.push(" RETURNING id, index");
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
                "INSERT INTO event (block_hash, block_number, block_timestamp, spec_version, block_status, trace_index, pallet_index, pallet_name, pallet_event_index, pallet_event_name, extrinsic_index, extrinsic_hash, phase, index, args_json) ",
            );
            query_builder.push_values(event_row_chunk, |mut query, event| {
                query
                    .push_bind(&event.block_hash)
                    .push_bind(event.block_number)
                    .push_bind(event.block_timestamp)
                    .push_bind(event.spec_version)
                    .push_bind(event.block_status)
                    .push_bind(event.trace_index)
                    .push_bind(event.pallet_index)
                    .push_bind(&event.pallet_name)
                    .push_bind(event.pallet_event_index)
                    .push_bind(&event.pallet_event_name)
                    .push_bind(event.extrinsic_index)
                    .push_bind(event.extrinsic_hash)
                    .push_bind(&event.phase)
                    .push_bind(event.index)
                    .push_bind(&event.args_json);
            });
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
        block_timestamp: u64,
        spec_version: u32,
        block_status: BlockStatus,
        extrinsic_id: i64,
        extrinsic_index: u32,
        extrinsic_hash: &[u8],
        parent_call_id: Option<i64>,
        nesting_index: Option<&str>,
        pallet_index: u8,
        pallet_name: &str,
        pallet_call_index: u8,
        pallet_call_name: &str,
        is_successful: bool,
        args: &JSONValue,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64> {
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO call (block_hash, block_number, block_timestamp, spec_version, block_status, extrinsic_id, extrinsic_index, extrinsic_hash, parent_call_id, nesting_index, pallet_index, pallet_name, pallet_call_index, pallet_call_name, is_successful, args_json)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp as i64)
            .bind(spec_version as i32)
            .bind(block_status)
            .bind(extrinsic_id)
            .bind(extrinsic_index as i32)
            .bind(extrinsic_hash)
            .bind(parent_call_id)
            .bind(nesting_index)
            .bind(pallet_index as i32)
            .bind(pallet_name)
            .bind(pallet_call_index as i32)
            .bind(pallet_call_name)
            .bind(is_successful)
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
    use sp_runtime::AccountId32;
    use std::fs;
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

    #[test_log::test(tokio::test)]
    async fn test_genesis_ingestion() -> Result<(), Box<dyn std::error::Error>> {
        let chainspec_path = "../_chainspecs/westend/sys/coretime-westend.json";
        let chainspec_json = fs::read_to_string(chainspec_path)?;
        let chainspec: Chainspec = serde_json::from_str(&chainspec_json)?;
        let postgres = get_test_postgres().await?;
        postgres.ingest_genesis(&chainspec).await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
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
            let hash = substrate_client.get_block_hash(number).await?;
            let header = substrate_client.get_block_header(&hash).await?;
            let timestamp = substrate_client.get_block_timestamp(&hash).await?;
            let last_runtime_upgrade = substrate_client
                .get_last_runtime_upgrade_info(&hash)
                .await?;
            let trace = substrate_client.get_block_trace(&hash).await?;
            let hash = hex::decode(hash)?;
            let mut tx = postgres.connection_pool.begin().await?;
            postgres
                .ingest_block(
                    &hash,
                    &header,
                    timestamp,
                    BlockStatus::Proposed,
                    &None,
                    last_runtime_upgrade.spec_version,
                    0,
                    0,
                    &AccountId32::new(Default::default()),
                    &mut tx,
                )
                .await?;
            postgres
                .ingest_block_logs(&hash, &header, BlockStatus::Proposed, &mut tx)
                .await?;
            postgres
                .ingest_block_trace(
                    &hash,
                    &header,
                    BlockStatus::Proposed,
                    last_runtime_upgrade.spec_version,
                    &trace,
                    &mut tx,
                )
                .await?;
            postgres.delete_error(&hash, &mut tx).await?;
            tx.commit().await?;
        }
        Ok(())
    }

    #[test_log::test(tokio::test)]
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
        let block_hash = substrate_client.get_block_hash(block_number).await?;
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

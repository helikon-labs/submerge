use frame_metadata::RuntimeMetadataPrefixed;
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;
use serde_json::Value as JsonValue;
use sp_runtime::AccountId32;
use sp_runtime::DigestItem;
use sqlx::{Postgres, Transaction};
use std::str::FromStr;
use submerge_base::types::submerge::BlockTrace as SubmergeBlockTrace;
use submerge_base::types::submerge::BlockTraces;
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::block_trace::{
    BlockTrace as SubstrateBlockTrace, StorageMethod,
};
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_util::substrate::storage::get_storage_plain_key;

use crate::types::metadata::Metadata;
use crate::types::metadata::Pallet;
use crate::types::Event;
use crate::types::Extrinsic;

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
        metadata_json: &JsonValue,
        metadata: &Metadata,
    ) -> anyhow::Result<()>;
    async fn ingest_metadata_pallet(
        &self,
        spec_version: u32,
        pallet: &Pallet,
    ) -> anyhow::Result<()>;
    async fn get_pallet_index_by_name(&self, name: &str) -> anyhow::Result<Option<u8>>;
    async fn get_pallet_call_index_by_name(
        &self,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>>;
    async fn get_pallet_event_index_by_name(
        &self,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>>;
    async fn get_genesis_record_count(&self) -> anyhow::Result<u64>;
    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()>;
    async fn get_next_block_number(&self, min: u64, max: u64) -> anyhow::Result<u64>;
    #[cfg(test)]
    async fn get_error_count(&self) -> anyhow::Result<u32>;
    async fn save_error(
        &self,
        block_hash: &[u8],
        block_number: u64,
        description: &str,
    ) -> anyhow::Result<()>;
    async fn delete_error(
        &self,
        block_hash: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_block(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        timestamp: u64,
        is_finalized: bool,
        spec_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        author_account_id: &AccountId32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn get_block_traces_by_number(
        &self,
        block_number: u64,
    ) -> anyhow::Result<Vec<BlockTraces>>;
    async fn get_block_traces_by_hash(
        &self,
        block_hash: &[u8],
    ) -> anyhow::Result<Option<BlockTraces>>;
    async fn block_trace_exists(&self, block_hash: &[u8]) -> anyhow::Result<bool>;
    async fn ingest_block_trace(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        is_finalized: bool,
        spec_version: u32,
        trace: &SubstrateBlockTrace,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn ingest_block_logs(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        is_finalized: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_extrinsic(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        extrinsic: &Extrinsic,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_event(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        event: &Event,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_call(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
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
        args: &JsonValue,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64>;

    async fn ingest_genesis_item(
        tx: &mut Transaction<'_, Postgres>,
        key: &str,
        value: &str,
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
        metadata_json: &JsonValue,
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
        .execute(&self.connection_pool)
        .await?;
        for pallet in metadata.pallets.iter() {
            self.ingest_metadata_pallet(spec_version, pallet).await?;
        }
        Ok(())
    }

    async fn ingest_metadata_pallet(
        &self,
        spec_version: u32,
        pallet: &Pallet,
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
        .execute(&self.connection_pool)
        .await?;
        for event in pallet.events.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_pallet_event (spec_version, pallet_index, pallet_name, index, name)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
            .bind(event.index as i32)
            .bind(&event.name)
            .execute(&self.connection_pool)
            .await?;
        }
        for constant in pallet.constants.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_pallet_constant (spec_version, pallet_index, pallet_name, index, name)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
            .bind(constant.index as i32)
            .bind(&constant.name)
            .execute(&self.connection_pool)
            .await?;
        }
        for call in pallet.calls.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_pallet_call (spec_version, pallet_index, pallet_name, index, name)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
            .bind(call.index as i32)
            .bind(&call.name)
            .execute(&self.connection_pool)
            .await?;
        }
        for storage_item in pallet.storage_items.iter() {
            let key = get_storage_plain_key(&pallet.name, &storage_item.name);
            sqlx::query(
                r#"
                INSERT INTO metadata_pallet_storage_item (spec_version, pallet_index, pallet_name, index, name, key)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
            .bind(storage_item.index as i32)
            .bind(&storage_item.name)
            .bind(&key)
            .execute(&self.connection_pool)
            .await?;
        }
        for error in pallet.errors.iter() {
            sqlx::query(
                r#"
                INSERT INTO metadata_pallet_error (spec_version, pallet_index, pallet_name, index, name)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(spec_version as i32)
            .bind(pallet.index as i32)
            .bind(&pallet.name)
            .bind(error.index as i32)
            .bind(&error.name)
            .execute(&self.connection_pool)
            .await?;
        }
        Ok(())
    }

    async fn get_pallet_index_by_name(&self, name: &str) -> anyhow::Result<Option<u8>> {
        let maybe_row: Option<(i32,)> =
            sqlx::query_as("SELECT index FROM metadata_pallet WHERE name = $1")
                .bind(name)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(maybe_row.map(|row| row.0 as u8))
    }

    async fn get_pallet_call_index_by_name(
        &self,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>> {
        let maybe_row: Option<(i32,)> = sqlx::query_as(
            "SELECT index FROM metadata_pallet_call WHERE pallet_index = $1 and name = $2",
        )
        .bind(pallet_index as i32)
        .bind(name)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_row.map(|row| row.0 as u8))
    }

    async fn get_pallet_event_index_by_name(
        &self,
        pallet_index: u8,
        name: &str,
    ) -> anyhow::Result<Option<u8>> {
        let maybe_row: Option<(i32,)> = sqlx::query_as(
            "SELECT index FROM metadata_pallet_event WHERE pallet_index = $1 and name = $2",
        )
        .bind(pallet_index as i32)
        .bind(name)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_row.map(|row| row.0 as u8))
    }

    async fn get_genesis_record_count(&self) -> anyhow::Result<u64> {
        let record_count: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT key) FROM genesis")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(record_count.0 as u64)
    }

    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()> {
        let mut tx = self.connection_pool.begin().await?;
        for (key, value) in chainspec.genesis.raw.top.iter() {
            Self::ingest_genesis_item(
                &mut tx,
                key.trim_start_matches("0x"),
                value.trim_start_matches("0x"),
            )
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn get_next_block_number(&self, min: u64, max: u64) -> anyhow::Result<u64> {
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT MAX(block_number) FROM trace WHERE block_number >= $1 AND block_number <= $2",
        )
        .bind(min as i64)
        .bind(max as i64)
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
        description: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO error (block_hash, block_number, description)
            VALUES ($1, $2, $3)
            ON CONFLICT(block_hash) DO UPDATE
            SET description = EXCLUDED.description, created_at = now()
        "#,
        )
        .bind(block_hash)
        .bind(block_number as i64)
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

    async fn ingest_block(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        timestamp: u64,
        is_finalized: bool,
        spec_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        author_account_id: &AccountId32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let parent_hash = hex::decode(&header.parent_hash)?;
        let state_root = hex::decode(&header.state_root)?;
        let extrinsic_root = hex::decode(&header.extrinsics_root)?;
        let author_account_id: &[u8; 32] = author_account_id.as_ref();
        sqlx::query(
            r#"
                INSERT INTO block (hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, is_finalized, extrinsic_count, event_count, author_account_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (hash) DO NOTHING
                "#,
        )
            .bind(hash)
            .bind(&parent_hash)
            .bind(&state_root)
            .bind(&extrinsic_root)
            .bind(header.get_number()? as i64)
            .bind(timestamp as i64)
            .bind(spec_version as i32)
            .bind(is_finalized)
            .bind(extrinsic_count as i32)
            .bind(event_count as i32)
            .bind(author_account_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn get_block_traces_by_number(
        &self,
        block_number: u64,
    ) -> anyhow::Result<Vec<BlockTraces>> {
        let block_hash_rows: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT DISTINCT block_hash FROM trace WHERE block_number = $1")
                .bind(block_number as i64)
                .fetch_all(&self.connection_pool)
                .await?;
        let mut result = vec![];
        for block_hash_row in block_hash_rows.iter() {
            if let Some(block_traces) = self.get_block_traces_by_hash(&block_hash_row.0).await? {
                result.push(block_traces);
            }
        }
        Ok(result)
    }

    async fn get_block_traces_by_hash(
        &self,
        block_hash: &[u8],
    ) -> anyhow::Result<Option<BlockTraces>> {
        #[allow(clippy::type_complexity)]
        let rows: Vec<(Vec<u8>, i64, i32, bool, i32, String, String, String, String, Option<String>)> = sqlx::query_as("SELECT block_parent_hash, block_number, spec_version, is_finalized, index, key, value, ext_id, method, parent_id FROM trace WHERE block_hash = $1 ORDER BY index ASC")
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;

        if let Some(first_row) = rows.first() {
            let block_hash_hex = format!("0x{}", hex::encode(block_hash));
            let block_parent_hash_hex = format!("0x{}", hex::encode(&first_row.0));
            let mut block_traces = BlockTraces {
                block_hash: block_hash_hex,
                block_parent_hash: block_parent_hash_hex,
                block_number: first_row.1 as u64,
                spec_version: first_row.2 as u32,
                is_finalized: first_row.3,
                traces: vec![],
            };
            for row in rows.iter() {
                block_traces.traces.push(SubmergeBlockTrace {
                    index: row.4 as u32,
                    key: row.5.clone(),
                    value: row.6.clone(),
                    ext_id: row.7.clone(),
                    method: StorageMethod::from_str(&row.8)?,
                    parent_id: row.9.clone(),
                })
            }
            Ok(Some(block_traces))
        } else {
            Ok(None)
        }
    }

    async fn block_trace_exists(&self, block_hash: &[u8]) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT index)
            FROM trace
            WHERE block_hash = $1
            "#,
        )
        .bind(block_hash)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    async fn ingest_block_trace(
        &self,
        hash: &[u8],
        header: &BlockHeader,
        is_finalized: bool,
        spec_version: u32,
        trace: &SubstrateBlockTrace,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let parent_hash = hex::decode(&header.parent_hash)?;
        for (trace_index, event) in trace.events.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO trace (block_hash, block_parent_hash, block_number, spec_version, is_finalized, index, key, value, ext_id, method, parent_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (block_hash, block_number, index) DO NOTHING
                "#,
            )
                .bind(hash)
                .bind(&parent_hash)
                .bind(header.get_number()? as i64)
                .bind(spec_version as i32)
                .bind(is_finalized)
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
        is_finalized: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
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
            sqlx::query(
                r#"
                INSERT INTO log (block_hash, block_number, is_finalized, index, type, engine, data)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (block_hash, index) DO NOTHING
                "#,
            )
            .bind(hash)
            .bind(header.get_number()? as i64)
            .bind(is_finalized)
            .bind(index as i32)
            .bind(ty)
            .bind(engine)
            .bind(data)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    async fn ingest_extrinsic(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        extrinsic: &Extrinsic,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64> {
        let (signer, signature, extra) = if let Some(signature) = &extrinsic.signature {
            (
                Some(Encode::encode(&signature.signer)),
                Some(Encode::encode(&signature.signature)),
                signature.extra.clone(),
            )
        } else {
            (None, None, None)
        };
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO extrinsic (block_hash, block_number, block_timestamp, spec_version, is_finalized, trace_index, hash, index, version, signer, signature, extra, is_successful)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp as i64)
            .bind(spec_version as i32)
            .bind(is_finalized)
            .bind(extrinsic.trace_index.map(|i| i as i32))
            .bind(extrinsic.hash)
            .bind(extrinsic.index as i32)
            .bind(extrinsic.version as i32)
            .bind(signer)
            .bind(signature)
            .bind(extra)
            .bind(extrinsic.is_successful)
            .fetch_one(&mut **tx)
            .await?;
        Ok(row.0)
    }

    async fn ingest_event(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        event: &Event,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64> {
        let (phase, extrinsic_index) = match &event.phase {
            frame_system::Phase::ApplyExtrinsic(extrinsic_index) => {
                ("ApplyExtrinsic", Some(extrinsic_index))
            }
            frame_system::Phase::Finalization => ("Finalization", None),
            frame_system::Phase::Initialization => ("Initialization", None),
        };
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO event (block_hash, block_number, block_timestamp, spec_version, is_finalized, trace_index, pallet_index, pallet_name, pallet_event_index, pallet_event_name, extrinsic_index, phase, index, args_json)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp as i64)
            .bind(spec_version as i32)
            .bind(is_finalized)
            .bind(event.trace_index as i32)
            .bind(event.pallet_index as i32)
            .bind(&event.pallet_name)
            .bind(event.pallet_event_index as i32)
            .bind(&event.pallet_event_name)
            .bind(extrinsic_index.map(|e| *e as i32))
            .bind(phase)
            .bind(event.index as i32)
            .bind(&event.args)
            .fetch_one(&mut **tx)
            .await?;
        Ok(row.0)
    }

    async fn ingest_call(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
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
        args: &JsonValue,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i64> {
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO call (block_hash, block_number, block_timestamp, spec_version, is_finalized, extrinsic_id, extrinsic_index, extrinsic_hash, parent_call_id, nesting_index, pallet_index, pallet_name, pallet_call_index, pallet_call_name, is_successful, args_json)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp as i64)
            .bind(spec_version as i32)
            .bind(is_finalized)
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
    use crate::persistence::{CrystalPostgreSQLStorage, PostgreSQLStorage};
    use sp_runtime::AccountId32;
    use std::fs;
    use submerge_base::{
        args::{PostgreSQLArgs, RPCArgs},
        types::substrate::chainspec::Chainspec,
    };
    use submerge_substrate_client::SubstrateClient;

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
        let args = RPCArgs {
            rpc_url: "wss://rpc.helikon.io/coretime-westend-dev".to_owned(),
            rpc_connection_timeout_secs: 30,
            rpc_request_timeout_secs: 30,
        };
        let substrate_client = SubstrateClient::new(&args).await?;
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
                    true,
                    last_runtime_upgrade.spec_version,
                    0,
                    0,
                    &AccountId32::new(Default::default()),
                    &mut tx,
                )
                .await?;
            postgres
                .ingest_block_logs(&hash, &header, true, &mut tx)
                .await?;
            postgres
                .ingest_block_trace(
                    &hash,
                    &header,
                    true,
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
        let args = RPCArgs {
            rpc_url: "wss://rpc.helikon.io/coretime-westend-dev".to_owned(),
            rpc_connection_timeout_secs: 30,
            rpc_request_timeout_secs: 30,
        };
        let substrate_client = SubstrateClient::new(&args).await?;
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let block_hash = hex::decode(block_hash)?;
        let mut tx = postgres.connection_pool.begin().await?;
        postgres.delete_error(&block_hash, &mut tx).await?;
        tx.commit().await?;
        let pre_trace_error_count = postgres.get_error_count().await?;
        postgres
            .save_error(&block_hash, block_number, "error_description")
            .await?;
        let post_trace_error_count = postgres.get_error_count().await?;
        assert_eq!(post_trace_error_count, pre_trace_error_count + 1);
        Ok(())
    }
}

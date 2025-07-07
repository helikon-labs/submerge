use frame_metadata::RuntimeMetadataPrefixed;
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;
use serde_json::Value as JsonValue;
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
use submerge_base::types::substrate::Signature;
use submerge_persistence::postgres::PostgreSQLStorage;

pub(crate) trait CrystalPostgreSQLStorage {
    async fn get_metadata_prefixed(
        &self,
        spec_version: u32,
    ) -> anyhow::Result<Option<RuntimeMetadataPrefixed>>;
    async fn ingest_metadata_prefixed(
        &self,
        spec_version: u32,
        version: u32,
        metadata_bytes: &[u8],
        metadata_json: &JsonValue,
    ) -> anyhow::Result<()>;
    async fn get_genesis_record_count(&self) -> anyhow::Result<u64>;
    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()>;
    async fn get_next_block_number(&self, min: u64, max: u64) -> anyhow::Result<u64>;
    #[cfg(test)]
    async fn get_trace_error_count(&self) -> anyhow::Result<u32>;
    async fn save_trace_error(
        &self,
        block_hash: &[u8],
        block_number: u64,
        description: &str,
    ) -> anyhow::Result<()>;
    async fn delete_trace_error(
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
        trace_index: Option<u32>,
        pallet_index: u8,
        pallet_name: &str,
        call_index: u8,
        call_name: &str,
        hash: &[u8],
        index: u32,
        version: u8,
        signature: &Option<Signature>,
        is_successful: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_event(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        trace_index: u32,
        pallet_index: u8,
        pallet_name: &str,
        event_index: u8,
        event_name: &str,
        extrinsic_index: Option<u32>,
        phase: &str,
        index: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;

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
    async fn get_metadata_prefixed(
        &self,
        spec_version: u32,
    ) -> anyhow::Result<Option<RuntimeMetadataPrefixed>> {
        let maybe_metadata_bytes: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT metadata_prefixed_bytes FROM metadata WHERE spec_version = $1")
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

    async fn ingest_metadata_prefixed(
        &self,
        spec_version: u32,
        version: u32,
        metadata_prefixed_bytes: &[u8],
        metadata_prefixed_json: &JsonValue,
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
            INSERT INTO metadata (spec_version, version, metadata_prefixed_bytes, metadata_prefixed_json)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(spec_version as i32)
        .bind(version as i32)
        .bind(metadata_prefixed_bytes)
        .bind(metadata_prefixed_json)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    async fn get_genesis_record_count(&self) -> anyhow::Result<u64> {
        let record_count: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT key) FROM genesis")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(record_count.0 as u64)
    }

    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()> {
        log::info!("🔽 Processing genesis from chainspec file.");
        if self.get_genesis_record_count().await? > 0 {
            log::info!("🔁 Genesis had already been processed.");
            return Ok(());
        }
        let mut tx = self.connection_pool.begin().await?;
        for (key, value) in chainspec.genesis.raw.top.iter() {
            Self::ingest_genesis_item(&mut tx, key, value).await?;
        }
        tx.commit().await?;
        log::info!(
            "✅ Processed {} storage items from the chainspec file.",
            chainspec.genesis.raw.top.len()
        );
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
    async fn get_trace_error_count(&self) -> anyhow::Result<u32> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT block_number) FROM trace_error")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(row.0 as u32)
    }

    async fn save_trace_error(
        &self,
        block_hash: &[u8],
        block_number: u64,
        description: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO trace_error (block_hash, block_number, description)
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

    async fn delete_trace_error(
        &self,
        block_hash: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM trace_error WHERE block_hash = $1")
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
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let parent_hash = hex::decode(&header.parent_hash)?;
        let state_root = hex::decode(&header.state_root)?;
        let extrinsic_root = hex::decode(&header.extrinsics_root)?;
        sqlx::query(
            r#"
                INSERT INTO block (hash, parent_hash, state_root, extrinsic_root, number, timestamp, spec_version, is_finalized, extrinsic_count, event_count)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        trace_index: Option<u32>,
        pallet_index: u8,
        pallet_name: &str,
        call_index: u8,
        call_name: &str,
        hash: &[u8],
        index: u32,
        version: u8,
        signature: &Option<Signature>,
        is_successful: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let (signer, signature, era, nonce, tip, extra) = if let Some(signature) = signature {
            (
                Some(Encode::encode(&signature.signer)),
                Some(Encode::encode(&signature.signature)),
                Some(Encode::encode(&signature.era)),
                Some(signature.nonce),
                Some(signature.tip),
                Some(signature.extra),
            )
        } else {
            (None, None, None, None, None, None)
        };
        sqlx::query(
            r#"
            INSERT INTO extrinsic (block_hash, block_number, block_timestamp, spec_version, is_finalized, trace_index, pallet_index, pallet_name, call_index, call_name, hash, index, version, nonce, signer, signature, era, tip, extra, is_successful, params)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, null)
            ON CONFLICT (block_hash, block_number, index) DO NOTHING
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp as i64)
            .bind(spec_version as i32)
            .bind(is_finalized)
            .bind(trace_index.map(|i| i as i32))
            .bind(pallet_index as i32)
            .bind(pallet_name)
            .bind(call_index as i32)
            .bind(call_name)
            .bind(hash)
            .bind(index as i32)
            .bind(version as i32)
            .bind(nonce.map(|e| e as i32))
            .bind(signer)
            .bind(signature)
            .bind(era)
            .bind(tip.map(|t| t.to_string()))
            .bind(extra.map(|e| e as i32))
            .bind(is_successful)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn ingest_event(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        trace_index: u32,
        pallet_index: u8,
        pallet_name: &str,
        event_index: u8,
        event_name: &str,
        extrinsic_index: Option<u32>,
        phase: &str,
        index: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO event (block_hash, block_number, block_timestamp, spec_version, is_finalized, trace_index, pallet_index, pallet_name, event_index, event_name, extrinsic_index, phase, index, params)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, null)
            ON CONFLICT (block_hash, block_number, index) DO NOTHING
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp as i64)
            .bind(spec_version as i32)
            .bind(is_finalized)
            .bind(trace_index as i32)
            .bind(pallet_index as i32)
            .bind(pallet_name)
            .bind(event_index as i32)
            .bind(event_name)
            .bind(extrinsic_index.map(|e| e as i32))
            .bind(phase)
            .bind(index as i32)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::{CrystalPostgreSQLStorage, PostgreSQLStorage};
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
            ws_rpc_url: "wss://rpc.helikon.io/coretime-westend-dev".to_owned(),
            http_rpc_url: "https://rpc.helikon.io/coretime-westend-dev".to_owned(),
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
            postgres.delete_trace_error(&hash, &mut tx).await?;
            tx.commit().await?;
        }
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_trace_error() -> Result<(), Box<dyn std::error::Error>> {
        let postgres = get_test_postgres().await?;
        let block_number = 100;
        let args = RPCArgs {
            ws_rpc_url: "wss://rpc.helikon.io/coretime-westend-dev".to_owned(),
            http_rpc_url: "https://rpc.helikon.io/coretime-westend-dev".to_owned(),
            rpc_connection_timeout_secs: 30,
            rpc_request_timeout_secs: 30,
        };
        let substrate_client = SubstrateClient::new(&args).await?;
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let block_hash = hex::decode(block_hash)?;
        let mut tx = postgres.connection_pool.begin().await?;
        postgres.delete_trace_error(&block_hash, &mut tx).await?;
        tx.commit().await?;
        let pre_trace_error_count = postgres.get_trace_error_count().await?;
        postgres
            .save_trace_error(&block_hash, block_number, "error_description")
            .await?;
        let post_trace_error_count = postgres.get_trace_error_count().await?;
        assert_eq!(post_trace_error_count, pre_trace_error_count + 1);
        Ok(())
    }
}

use std::cmp::min;
use std::sync::{Arc, LazyLock};

use crate::api::legacy::LegacyDecodeAPIClient;
use crate::persistence::CrystalPostgreSQLStorage;
use crate::types::BlockStatus;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::account_id::AccountId;
use submerge_base::types::substrate::block::BlockHeader;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::{RPCConfig, SubstrateClient};
use submerge_util::string::truncate_hash;
use tokio::sync::RwLock;
use uuid::Uuid as UUID;

mod event;
mod extrinsic;
mod metadata_cache;
mod weight;

static SESSION_VALIDATORS_CACHE: LazyLock<RwLock<(u32, Vec<AccountId>)>> =
    LazyLock::new(|| RwLock::new((0, Vec::new())));

pub struct BlockProcessor {
    worker_id: UUID,
    postgres: Arc<PostgreSQLStorage>,
    substrate_client: SubstrateClient,
    legacy_decode_api_client: Option<LegacyDecodeAPIClient>,
}

impl BlockProcessor {
    pub async fn new(
        worker_id: UUID,
        postgres: Arc<PostgreSQLStorage>,
        rpc_config: &RPCConfig,
        legacy_decode_api_url: &Option<String>,
    ) -> anyhow::Result<Self> {
        let substrate_client = SubstrateClient::new(rpc_config).await?;
        let legacy_decode_api_client = if let Some(url) = legacy_decode_api_url {
            Some(LegacyDecodeAPIClient::new(url)?)
        } else {
            None
        };
        Ok(Self {
            worker_id,
            postgres,
            substrate_client,
            legacy_decode_api_client,
        })
    }

    pub async fn save_block_error(
        &self,
        block_hash: &[u8],
        block_number: u64,
        status: BlockStatus,
        description: &str,
    ) -> anyhow::Result<()> {
        self.postgres
            .save_error(block_hash, block_number, status, description)
            .await
    }

    async fn get_actual_finalized_block_range(
        &self,
        maybe_start_block_number: Option<u64>,
        maybe_end_block_number: Option<u64>,
        scan: bool,
    ) -> anyhow::Result<(u64, u64)> {
        let start_block_number = maybe_start_block_number.unwrap_or(0);
        let finalized_block_hash = self.substrate_client.get_finalized_block_hash().await?;
        let finalized_block_number = self
            .substrate_client
            .get_block_header(&finalized_block_hash)
            .await?
            .get_number()?;
        let end_block_number = min(
            maybe_end_block_number.unwrap_or(finalized_block_number),
            finalized_block_number,
        );
        let start_block_number = if scan {
            start_block_number
        } else {
            self.postgres
                .get_next_block_number(start_block_number, end_block_number, BlockStatus::Finalized)
                .await?
        };
        Ok((start_block_number, end_block_number))
    }

    pub async fn process_finalized_blocks_in_range(
        &self,
        stop_on_error: bool,
        skip_traces: bool,
        scan: bool,
        reindex: bool,
        maybe_start_block_number: Option<u64>,
        maybe_end_block_number: Option<u64>,
    ) -> anyhow::Result<()> {
        let (start_block_number, end_block_number) = self
            .get_actual_finalized_block_range(
                maybe_start_block_number,
                maybe_end_block_number,
                scan,
            )
            .await?;
        log::info!("⚙️ Process finalized blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            let hash_hex = self.substrate_client.get_block_hash(number).await?;
            let hash = hex::decode(&hash_hex)?;
            log::info!(
                "🔧 Processing finalized block [{number}][0x{}]. Target {end_block_number}.",
                truncate_hash(&hash_hex),
            );
            match self
                .process_block(
                    skip_traces,
                    reindex,
                    &hash_hex,
                    number,
                    BlockStatus::Finalized,
                )
                .await
            {
                Ok(_) => crate::metrics::processed_finalized_block_number()
                    .with_label_values(&[&self.worker_id.to_string()])
                    .set(number as i64),
                Err(error) => {
                    log::error!("❌ Error while processing finalized block {number}: {error:?}");
                    self.save_block_error(
                        &hash,
                        number,
                        BlockStatus::Finalized,
                        &error.to_string(),
                    )
                    .await?;
                    if stop_on_error {
                        return Err(error);
                    }
                }
            }
        }
        log::info!(
            "✅ Completed processing finalized blocks {start_block_number}-{end_block_number}."
        );
        Ok(())
    }

    async fn process_block_0(
        &self,
        block_hash: &[u8],
        block_header: &BlockHeader,
        spec_version: u32,
        status: BlockStatus,
    ) -> anyhow::Result<()> {
        let mut tx = self.postgres.connection_pool.begin().await?;
        self.postgres
            .ingest_block(
                block_hash,
                block_header,
                0,
                status,
                &None,
                spec_version,
                0,
                0,
                &AccountId::new([0u8; 32]),
                &mut tx,
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn prune_other_blocks(
        &self,
        block_number: u64,
        block_hash: &[u8],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let blocks = self
            .postgres
            .get_blocks_by_number_with_tx(block_number, tx)
            .await?;
        for block in blocks.iter() {
            if block.hash != block_hash {
                log::info!(
                    "✂️ Prune block [{block_number}][0x{}].",
                    truncate_hash(&hex::encode(&block.hash)),
                );
                self.postgres
                    .update_block_status(&block.hash, BlockStatus::Pruned, tx)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn get_finalized_block_hash_hex(&self, block_number: u64) -> anyhow::Result<String> {
        self.substrate_client.get_block_hash(block_number).await
    }

    #[allow(clippy::cognitive_complexity)]
    pub async fn process_block(
        &self,
        skip_traces: bool,
        reindex: bool,
        block_hash_hex: &str,
        block_number: u64,
        status: BlockStatus,
    ) -> anyhow::Result<()> {
        let block_hash = hex::decode(block_hash_hex)?;
        let mut tx = self.postgres.connection_pool.begin().await?;
        if let Some(block_row) = self.postgres.get_block_by_hash(&block_hash).await? {
            log::info!(
                "👍 Block [{block_number}][{}] had already been processed.",
                truncate_hash(block_hash_hex)
            );
            if reindex {
                log::info!(
                    "🗑️  Deleting block [{block_number}][{}] and its traces for reindexing.",
                    truncate_hash(block_hash_hex)
                );
                self.postgres
                    .delete_block_and_traces_by_hash(&block_hash, &mut tx)
                    .await?;
            } else {
                if block_row.status != status && status == BlockStatus::Finalized {
                    let start_time = std::time::Instant::now();
                    log::info!(
                        "🔁 Update block [{block_number}][0x{}] status: {} ➡️ {status}",
                        truncate_hash(block_hash_hex),
                        block_row.status,
                    );
                    self.postgres
                        .update_block_status(&block_hash, status, &mut tx)
                        .await?;
                    self.prune_other_blocks(block_number, &block_hash, &mut tx)
                        .await?;
                    let elapsed_time_ms = start_time.elapsed().as_millis();
                    crate::metrics::block_status_update_time_ms()
                        .with_label_values(&[&self.worker_id.to_string()])
                        .observe(elapsed_time_ms as f64);
                }
                tx.commit().await?;
                return Ok(());
            }
        }
        let start_time = std::time::Instant::now();
        let block_header = self
            .substrate_client
            .get_block_header(block_hash_hex)
            .await?;
        let spec_version = self
            .substrate_client
            .get_last_runtime_upgrade_info(block_hash_hex)
            .await?
            .spec_version;
        if block_number == 0 {
            self.process_block_0(&block_hash, &block_header, spec_version, status)
                .await?;
            return Ok(());
        }
        let metadata = self.get_metadata(&block_hash, spec_version).await?;
        let author_account_id = {
            let validator_index = block_header.get_validator_index()?;
            let session_index = self
                .substrate_client
                .get_current_session_index(block_hash_hex)
                .await?;
            let mut session_validators_cache = SESSION_VALIDATORS_CACHE.write().await;
            let validator_account_ids = {
                if session_validators_cache.0 != session_index
                    || session_validators_cache.1.is_empty()
                {
                    let validator_account_ids = self
                        .substrate_client
                        .get_active_validator_account_ids(block_hash_hex)
                        .await?;
                    session_validators_cache.0 = session_index;
                    session_validators_cache.1 = validator_account_ids;
                }
                &session_validators_cache.1
            };
            let validator_index = validator_index % validator_account_ids.len() as u32;
            if let Some(author_account_id) = validator_account_ids.get(validator_index as usize) {
                *author_account_id
            } else {
                anyhow::bail!("Author validator was not found at index {validator_index}.");
            }
        };
        let block_timestamp = self
            .substrate_client
            .get_block_timestamp(block_hash_hex)
            .await?;
        let (events, extrinsics, weight) = if skip_traces {
            let event_bytes = self
                .substrate_client
                .get_block_event_bytes(block_hash_hex)
                .await?;
            let events = self
                .get_events_from_event_bytes(&block_hash, spec_version, &metadata, event_bytes)
                .await?;
            let extrinsics = self
                .get_extrinsics(&block_hash, spec_version, &metadata, &events)
                .await?;
            let weight = self
                .get_block_weight_from_rpc(&block_hash, spec_version, &metadata)
                .await?;
            (events, extrinsics, weight)
        } else {
            let trace = self
                .substrate_client
                .get_block_trace(block_hash_hex)
                .await?;
            self.postgres
                .ingest_block_trace(&block_hash, &block_header, spec_version, &trace, &mut tx)
                .await?;
            let event_count = trace.get_event_count()?;
            let events = self
                .get_events_from_trace(&block_hash, spec_version, &metadata, &trace)
                .await?;
            if event_count != events.len() as u32 {
                anyhow::bail!(
                    "❌ Expected event count {event_count} is not equal to decoded event count {}.",
                    events.len()
                );
            }

            let extrinsic_count = trace.get_extrinsic_count()?;
            let extrinsics = self
                .get_extrinsics_from_trace(&block_hash, spec_version, &metadata, &trace, &events)
                .await?;
            if extrinsic_count != extrinsics.len() as u32 {
                anyhow::bail!(
                    "❌ Expected extrinsic count {extrinsic_count} is not equal to decoded event count {}.",
                    extrinsics.len()
                );
            }
            let weight = self
                .get_block_weight_from_trace(&block_hash, spec_version, &metadata, &trace)
                .await?;
            (events, extrinsics, weight)
        };
        log::info!("Decoded {} events.", events.len());
        log::info!("Decoded {} extrinsics.", extrinsics.len());

        // persist block, events, and extrinsics
        if status == BlockStatus::Finalized {
            self.prune_other_blocks(block_number, &block_hash, &mut tx)
                .await?;
        }
        self.postgres
            .ingest_block(
                &block_hash,
                &block_header,
                block_timestamp,
                status,
                &weight,
                spec_version,
                extrinsics.len() as u32,
                events.len() as u32,
                &author_account_id,
                &mut tx,
            )
            .await?;
        self.postgres
            .ingest_block_logs(&block_hash, &block_header, &mut tx)
            .await?;
        log::info!("Persisted block and logs.");
        self.process_events(
            &block_hash,
            &block_header,
            block_timestamp,
            spec_version,
            status,
            &events,
            &extrinsics,
            &mut tx,
        )
        .await?;
        log::info!("Persisted {} events.", events.len());
        self.process_extrinsics(
            &block_hash,
            &block_header,
            block_timestamp,
            spec_version,
            status,
            &extrinsics,
            &mut tx,
        )
        .await?;
        log::info!("Persisted {} extrinsics.", extrinsics.len());
        self.postgres.delete_error(&block_hash, &mut tx).await?;
        tx.commit().await?;
        let log_emoji = match status {
            BlockStatus::Proposed => "🟦",
            BlockStatus::Pruned => "⬜",
            BlockStatus::Finalized => "🟩",
        };
        let elapsed_time_ms = start_time.elapsed().as_millis();
        crate::metrics::block_processing_time_ms()
            .with_label_values(&[&self.worker_id.to_string()])
            .observe(elapsed_time_ms as f64);
        log::info!(
            "{log_emoji} Processed {status} block [{block_number}][0x{}] in {elapsed_time_ms} ms.",
            truncate_hash(block_hash_hex),
        );
        Ok(())
    }
}

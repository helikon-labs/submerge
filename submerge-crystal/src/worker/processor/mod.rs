use std::cmp::min;
use std::sync::{Arc, LazyLock};

use crate::api::legacy::LegacyDecodeAPIClient;
use crate::metadata_cache::{get_metadata, get_parsed_metadata};
use crate::persistence::CrystalPostgreSQLStorage;
use crate::types::metadata::util::{
    get_metadata_type_by_id, get_metadata_version, get_pallet_storage_item_type_by_name,
};
use crate::types::BlockStatus;
use crate::worker::WorkerError;
use anyhow::Context as _;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::account_id::AccountId;
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::multi_address::MultiAddress;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::{RPCConfig, SubstrateClient};
use submerge_util::string::truncate_hash;
use tokio::sync::RwLock;
use uuid::Uuid as UUID;

mod event;
mod extrinsic;
mod weight;

const TRANSACTION_LEVEL_KEY: &[u8] = b":transaction_level:";
const METADATA_VERSION_LEGACY_THRESHOLD: u32 = 14;
const ACCOUNT_ID_32_TYPE_PATH: &str = "sp_core::crypto::AccountId32";
const ACCOUNT_ID_20_TYPE_PATH: &str = "account::AccountId20";
const AUTHOR_INHERENT_PALLET_NAME: &str = "AuthorInherent";
const AUTHOR_STORAGE_ITEM_NAME: &str = "Author";
const SESSION_PALLET_NAME: &str = "Session";
const VALIDATORS_STORAGE_ITEM_NAME: &str = "Validators";

static SESSION_VALIDATORS_CACHE: LazyLock<RwLock<(u32, Vec<MultiAddress>)>> =
    LazyLock::new(|| RwLock::new((0, Vec::new())));

fn validate_block_range(
    maybe_start_block_number: Option<u64>,
    maybe_end_block_number: Option<u64>,
) -> Result<(), WorkerError> {
    if let Some((start, end)) = maybe_start_block_number.zip(maybe_end_block_number) {
        if start > end {
            return Err(WorkerError::InvalidFinalizedRange(start, end));
        }
    }
    Ok(())
}

pub struct BlockProcessor {
    chain_name: String,
    worker_id: UUID,
    postgres: Arc<PostgreSQLStorage>,
    substrate_client: SubstrateClient,
    legacy_decode_api_client: Option<LegacyDecodeAPIClient>,
}

impl BlockProcessor {
    pub async fn new(
        chain_name: &str,
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
            chain_name: chain_name.to_string(),
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
        validate_block_range(maybe_start_block_number, maybe_end_block_number)?;
        let (start_block_number, end_block_number) = self
            .get_actual_finalized_block_range(
                maybe_start_block_number,
                maybe_end_block_number,
                scan,
            )
            .await?;
        tracing::info!("⚙️ Process finalized blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            let Some(hash_hex) = self.substrate_client.get_block_hash(number).await? else {
                anyhow::bail!("Finalized block {number} not found on the RPC node.");
            };
            let hash = hex::decode(&hash_hex)?;
            tracing::info!(
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
                Ok(_) => crate::metrics::processed_finalized_block_number()?
                    .with_label_values([&self.worker_id.to_string(), "finalized_range"].as_slice())
                    .set(number as i64),
                Err(error) => {
                    tracing::error!(
                        "❌ Error while processing finalized block {number}: {error:?}"
                    );
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
        tracing::info!(
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
                None,
                status,
                &None,
                spec_version,
                0,
                0,
                &None,
                &mut tx,
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn prune_other_blocks_with_number(
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
                tracing::info!(
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

    pub async fn get_block_hash_hex(&self, block_number: u64) -> anyhow::Result<Option<String>> {
        self.substrate_client.get_block_hash(block_number).await
    }

    async fn get_block_author(
        &self,
        block_hash_hex: &str,
        spec_version: u32,
        block_header: &BlockHeader,
    ) -> anyhow::Result<Option<MultiAddress>> {
        let block_hash = hex::decode(block_hash_hex)?;
        let is_nimbus = get_parsed_metadata(
            &block_hash,
            spec_version,
            &self.postgres,
            &self.substrate_client,
            &self.legacy_decode_api_client,
        )
        .await?
        .has_storage_item(AUTHOR_INHERENT_PALLET_NAME, AUTHOR_STORAGE_ITEM_NAME);
        let author_multi_address = if is_nimbus {
            self.substrate_client
                .get_nimbus_block_author(block_hash_hex)
                .await?
        } else if let Some(validator_index) = block_header.get_validator_index()? {
            let session_index = self
                .substrate_client
                .get_current_session_index(block_hash_hex)
                .await?;
            let mut session_validators_cache = SESSION_VALIDATORS_CACHE.write().await;
            let validator_addresses = {
                if session_validators_cache.0 != session_index
                    || session_validators_cache.1.is_empty()
                {
                    let metadata = get_metadata(
                        &block_hash,
                        spec_version,
                        &self.postgres,
                        &self.substrate_client,
                        &self.legacy_decode_api_client,
                    )
                    .await?;
                    let sequence_type_path =
                        if get_metadata_version(&metadata) < METADATA_VERSION_LEGACY_THRESHOLD {
                            ACCOUNT_ID_32_TYPE_PATH.to_string()
                        } else {
                            let session_validators_type = get_pallet_storage_item_type_by_name(
                                &metadata,
                                SESSION_PALLET_NAME,
                                VALIDATORS_STORAGE_ITEM_NAME,
                            )?
                            .ok_or(anyhow::Error::msg(format!(
                                "Session.Validators storage item not found in {} metadata.",
                                self.chain_name
                            )))?;
                            match &session_validators_type.ty.type_def {
                                scale_info::TypeDef::Sequence(sequence_type) => {
                                    let sequence_type = get_metadata_type_by_id(
                                        &metadata,
                                        sequence_type.type_param.id,
                                    )?
                                    .ok_or(anyhow::Error::msg(format!(
                                    "Session.Validators sequence type not found in {} metadata.",
                                    self.chain_name
                                )))?;
                                    sequence_type.ty.path.segments.join("::")
                                }
                                _ => anyhow::bail!(
                                    "Unexpected non-sequence type for Session.Validators: {:?}",
                                    session_validators_type.ty.type_def
                                ),
                            }
                        };
                    let validator_multi_addresses: Vec<MultiAddress> =
                        match sequence_type_path.as_str() {
                            ACCOUNT_ID_20_TYPE_PATH => self
                                .substrate_client
                                .get_active_validator_account_ids::<[u8; 20]>(block_hash_hex)
                                .await?
                                .iter()
                                .map(|address| MultiAddress::Address20(*address))
                                .collect(),
                            ACCOUNT_ID_32_TYPE_PATH => self
                                .substrate_client
                                .get_active_validator_account_ids::<AccountId>(block_hash_hex)
                                .await?
                                .iter()
                                .map(|account_id| MultiAddress::Id(*account_id))
                                .collect(),
                            _ => anyhow::bail!(
                            "Unexpected sequence type for Session.Validators: {sequence_type_path}"
                        ),
                        };
                    session_validators_cache.0 = session_index;
                    session_validators_cache.1 = validator_multi_addresses;
                }
                &session_validators_cache.1
            };
            let validator_index = validator_index % validator_addresses.len() as u32;
            if let Some(author_multi_address) = validator_addresses.get(validator_index as usize) {
                Some(author_multi_address.clone())
            } else {
                anyhow::bail!("Author validator was not found at index {validator_index}.");
            }
        } else {
            None
        };
        Ok(author_multi_address)
    }

    pub async fn process_block(
        &self,
        skip_traces: bool,
        reindex: bool,
        block_hash_hex: &str,
        block_number: u64,
        status: BlockStatus,
    ) -> anyhow::Result<()> {
        let block_hash = hex::decode(block_hash_hex)?;
        let truncated_block_hash = truncate_hash(block_hash_hex);
        let mut tx = self.postgres.connection_pool.begin().await?;
        if let Some(block_row) = self.postgres.get_block_by_hash(&block_hash).await? {
            tracing::info!(
                "👍 Block [{block_number}][{truncated_block_hash}] had already been processed."
            );
            if reindex {
                tracing::info!(
                    "🗑️  Deleting block [{block_number}][{truncated_block_hash}] and its traces for reindexing.",
                );
                self.postgres
                    .delete_block_and_traces_by_hash(&block_hash, &mut tx)
                    .await?;
            } else {
                if block_row.status != status && status == BlockStatus::Finalized {
                    let start_time = std::time::Instant::now();
                    tracing::info!(
                        "🔁 Update block [{block_number}][0x{truncated_block_hash}] status: {} ➡️ {status}",
                        block_row.status,
                    );
                    self.postgres
                        .update_block_status(&block_hash, status, &mut tx)
                        .await?;
                    self.prune_other_blocks_with_number(block_number, &block_hash, &mut tx)
                        .await?;
                    crate::metrics::block_status_update_time_ms()?
                        .with_label_values(&[&self.worker_id.to_string()])
                        .observe(start_time.elapsed().as_millis() as f64);
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
        let metadata = get_metadata(
            &block_hash,
            spec_version,
            &self.postgres,
            &self.substrate_client,
            &self.legacy_decode_api_client,
        )
        .await?;
        let parsed_metadata = get_parsed_metadata(
            &block_hash,
            spec_version,
            &self.postgres,
            &self.substrate_client,
            &self.legacy_decode_api_client,
        )
        .await?;
        let author_multi_address = self
            .get_block_author(block_hash_hex, spec_version, &block_header)
            .await?;
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
                // find storage item
                let storage_item = parsed_metadata
                    .pallets
                    .iter()
                    .flat_map(|pallet| &pallet.storage_items)
                    .find(|item| key.starts_with(&item.key_prefix));
                // check for known key
                let is_known_key = matches!(
                    key.as_slice(),
                    sp_storage::well_known_keys::CHILD_STORAGE_KEY_PREFIX
                        | sp_storage::well_known_keys::CODE
                        | sp_storage::well_known_keys::DEFAULT_CHILD_STORAGE_KEY_PREFIX
                        | sp_storage::well_known_keys::EXTRINSIC_INDEX
                        | sp_storage::well_known_keys::HEAP_PAGES
                        | sp_storage::well_known_keys::INTRABLOCK_ENTROPY
                        | TRANSACTION_LEVEL_KEY,
                );
                if storage_item.is_none() && !is_known_key {
                    tracing::warn!(
                        "Trace {trace_index} of block [{block_number}][0x{}] has unknown key: 0x{}",
                        truncate_hash(block_hash_hex),
                        event.data_wrapper.data.key
                    );
                }

                let (key_prefix, key_params) = if let Some(storage_item) = storage_item {
                    (
                        storage_item.key_prefix.as_slice(),
                        if key.len() > storage_item.key_prefix.len() {
                            key.get(storage_item.key_prefix.len()..)
                        } else {
                            None
                        },
                    )
                } else {
                    (key.as_slice(), None)
                };
                // ingest
                self.postgres
                    .ingest_block_trace(
                        &block_hash,
                        block_number,
                        spec_version,
                        trace_index as u32,
                        key_prefix,
                        key_params,
                        value.as_deref(),
                        &ext_id,
                        &event.data_wrapper.data.method.to_string(),
                        event.parent_id.as_deref(),
                        storage_item.map(|storage_item| storage_item.id),
                        is_known_key,
                        &mut tx,
                    )
                    .await?;
            }

            let event_count = trace.get_event_count()?;
            let events = self
                .get_events_from_trace(&block_hash, spec_version, &metadata, &trace)
                .await?;
            if event_count != events.len() as u32 {
                tracing::warn!(
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
            tracing::info!(
                block_number,
                "Processed and persisted {} traces.",
                trace.events.len()
            );
            (events, extrinsics, weight)
        };
        tracing::info!(block_number, "Decoded {} extrinsics.", extrinsics.len(),);
        tracing::info!(block_number, "Decoded {} events.", events.len(),);
        // persist block, events, and extrinsics
        if status == BlockStatus::Finalized {
            self.prune_other_blocks_with_number(block_number, &block_hash, &mut tx)
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
                &author_multi_address,
                &mut tx,
            )
            .await?;
        self.postgres
            .ingest_block_logs(&block_hash, &block_header, &mut tx)
            .await?;
        tracing::info!("Persisted block and logs.");
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
        tracing::info!("Persisted {} events.", events.len());
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
        tracing::info!("Persisted {} extrinsics.", extrinsics.len());
        self.postgres.delete_error(&block_hash, &mut tx).await?;
        tx.commit().await?;

        let log_emoji = match status {
            BlockStatus::Proposed => "🟦",
            BlockStatus::Pruned => "⬜",
            BlockStatus::Finalized => "🟩",
        };
        let elapsed_time_ms = start_time.elapsed().as_millis();
        crate::metrics::block_processing_time_ms()?
            .with_label_values(&[&self.worker_id.to_string()])
            .observe(elapsed_time_ms as f64);
        tracing::info!(
            "{log_emoji} Processed {status} block [{block_number}][0x{}] in {elapsed_time_ms} ms.",
            truncate_hash(block_hash_hex),
        );
        Ok(())
    }
}

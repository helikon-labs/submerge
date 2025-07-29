use std::cmp::min;
use std::sync::LazyLock;

use crate::api::legacy::LegacyDecodeAPIClient;
use crate::persistence::CrystalPostgreSQLStorage;
use crate::types::BlockStatus;
use sp_runtime::AccountId32;
use submerge_base::args::{PostgreSQLArgs, RPCArgs};
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;
use tokio::sync::RwLock;

pub(crate) mod decode;
mod event;
mod extrinsic;
mod metadata;

static SESSION_VALIDATORS_CACHE: LazyLock<RwLock<(u32, Vec<AccountId32>)>> =
    LazyLock::new(|| RwLock::new((0, Vec::new())));

pub struct BlockProcessor {
    postgres: PostgreSQLStorage,
    substrate_client: SubstrateClient,
    legacy_decode_api_client: Option<LegacyDecodeAPIClient>,
}

impl BlockProcessor {
    pub async fn new(
        postgres_args: &PostgreSQLArgs,
        rpc_args: &RPCArgs,
        legacy_decode_api_url: &Option<String>,
    ) -> anyhow::Result<Self> {
        let postgres = PostgreSQLStorage::new(postgres_args).await?;
        let substrate_client = SubstrateClient::new(rpc_args).await?;
        let legacy_decode_api_client = if let Some(url) = legacy_decode_api_url {
            Some(LegacyDecodeAPIClient::new(url)?)
        } else {
            None
        };
        Ok(Self {
            postgres,
            substrate_client,
            legacy_decode_api_client,
        })
    }

    pub async fn process_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()> {
        log::info!("🔽 Processing genesis from chainspec file.");
        if self.postgres.get_genesis_record_count().await? > 0 {
            log::info!("🔁 Genesis had already been processed.");
            return Ok(());
        }
        self.postgres.ingest_genesis(chainspec).await?;
        log::info!(
            "✅ Processed {} storage items from the chainspec file.",
            chainspec.genesis.raw.top.len()
        );
        Ok(())
    }

    pub async fn process_finalized_blocks_in_range(
        &self,
        scan: bool,
        stop_on_error: bool,
        skip_traces: bool,
        start_block_number: u64,
        end_block_number: u64,
    ) -> anyhow::Result<()> {
        let start_block_number = if scan {
            start_block_number
        } else {
            self.postgres
                .get_next_block_number(start_block_number, end_block_number, BlockStatus::Finalized)
                .await?
        };
        let finalized_block_hash = self.substrate_client.get_finalized_block_hash().await?;
        let finalized_block_number = self
            .substrate_client
            .get_block_header(&finalized_block_hash)
            .await?
            .get_number()?;
        let end_block_number = min(end_block_number, finalized_block_number);
        log::info!("⚙️ Process blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            log::info!("🔧 Processing block {number}. Target {end_block_number}.");
            let hash_hex = self.substrate_client.get_block_hash(number).await?;
            let hash = hex::decode(&hash_hex)?;
            match self
                .process_block(skip_traces, &hash_hex, number, BlockStatus::Finalized)
                .await
            {
                Ok(_) => (),
                Err(error) => {
                    log::error!("❌ Error while processing block {number}: {error:?}");
                    self.postgres
                        .save_error(&hash, number, &error.to_string())
                        .await?;
                    if stop_on_error {
                        return Err(error);
                    }
                }
            }
        }
        log::info!("✅ Completed processing blocks {start_block_number}-{end_block_number}.");
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
                spec_version,
                0,
                0,
                &AccountId32::new([0u8; 32]),
                &mut tx,
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    pub async fn process_block(
        &self,
        skip_traces: bool,
        block_hash_hex: &str,
        block_number: u64,
        status: BlockStatus,
    ) -> anyhow::Result<()> {
        let block_hash = hex::decode(block_hash_hex)?;
        if self.postgres.block_trace_exists(&block_hash).await? {
            log::info!("🔁 Block {block_number} had already been ingested.");
            return Ok(());
        }
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
        let metadata = self.get_metadata(block_hash_hex, spec_version).await?;
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
                author_account_id.clone()
            } else {
                anyhow::bail!("Author validator was not found at index {validator_index}.");
            }
        };
        let block_timestamp = self
            .substrate_client
            .get_block_timestamp(block_hash_hex)
            .await?;
        let mut tx = self.postgres.connection_pool.begin().await?;
        let (events, extrinsics) = if skip_traces {
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
            (events, extrinsics)
        } else {
            let trace = self
                .substrate_client
                .get_block_trace(block_hash_hex)
                .await?;
            self.postgres
                .ingest_block_trace(
                    &block_hash,
                    &block_header,
                    status,
                    spec_version,
                    &trace,
                    &mut tx,
                )
                .await?;
            let event_count = trace.get_event_count()?;
            let events = self
                .get_events_from_traces(&block_hash, spec_version, &metadata, &trace)
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
            (events, extrinsics)
        };
        log::info!("Decoded {} events.", events.len());
        log::info!("Decoded {} extrinsics.", extrinsics.len());

        // persist block, events, and extrinsics
        self.postgres
            .ingest_block(
                &block_hash,
                &block_header,
                block_timestamp,
                status,
                spec_version,
                extrinsics.len() as u32,
                events.len() as u32,
                &author_account_id,
                &mut tx,
            )
            .await?;
        self.postgres
            .ingest_block_logs(&block_hash, &block_header, status, &mut tx)
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
        // TODO if finalized, check pruned blocks :: set pruned where hash != my_hash && number == my_number
        tx.commit().await?;
        let log_emoji = match status {
            BlockStatus::Proposed => "🟦",
            BlockStatus::Pruned => "⬜",
            BlockStatus::Finalized => "🟩",
        };
        log::info!("{log_emoji} Processed {status} block {block_number}.");
        Ok(())
    }
}

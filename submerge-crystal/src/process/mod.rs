use crate::api::legacy::LegacyDecodeAPIClient;
use crate::persistence::CrystalPostgreSQLStorage;
use submerge_base::args::{PostgreSQLArgs, RPCArgs};
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;

mod decode;
mod event;
mod extrinsic;
mod metadata;
mod util;

pub struct BlockProcessor {
    postgres: PostgreSQLStorage,
    substrate_client: SubstrateClient,
    legacy_decode_api_client: LegacyDecodeAPIClient,
}

impl BlockProcessor {
    pub async fn new(postgres_args: &PostgreSQLArgs, rpc_args: &RPCArgs) -> anyhow::Result<Self> {
        let postgres = PostgreSQLStorage::new(postgres_args).await?;
        let substrate_client = SubstrateClient::new(rpc_args).await?;
        let legacy_decode_api_client = LegacyDecodeAPIClient::new()?;
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

    pub async fn process_blocks(
        &self,
        scan: bool,
        stop_on_error: bool,
        start_block_number: u64,
        end_block_number: u64,
    ) -> anyhow::Result<()> {
        let start_block_number = if scan {
            start_block_number
        } else {
            self.postgres
                .get_next_block_number(start_block_number, end_block_number)
                .await?
        };
        log::info!("⚙️ Process blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            log::info!("🔧 Processing block {number}. Target {end_block_number}.");
            let hash_hex = self.substrate_client.get_block_hash(number).await?;
            let hash = hex::decode(&hash_hex)?;
            match self.process_block(&hash_hex, number, true).await {
                Ok(_) => {
                    log::info!("✅ Processed block {number}.");
                }
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
        is_finalized: bool,
    ) -> anyhow::Result<()> {
        let mut tx = self.postgres.connection_pool.begin().await?;
        self.postgres
            .ingest_block(
                block_hash,
                block_header,
                0,
                is_finalized,
                spec_version,
                0,
                0,
                &mut tx,
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    async fn process_block(
        &self,
        block_hash_hex: &str,
        block_number: u64,
        is_finalized: bool,
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
            self.process_block_0(&block_hash, &block_header, spec_version, is_finalized)
                .await?;
            return Ok(());
        }
        let block_timestamp = self
            .substrate_client
            .get_block_timestamp(block_hash_hex)
            .await?;
        let metadata = self.get_metadata(block_hash_hex, spec_version).await?;
        let trace = self
            .substrate_client
            .get_block_trace(block_hash_hex)
            .await?;
        let mut tx = self.postgres.connection_pool.begin().await?;
        self.postgres
            .ingest_block_trace(
                &block_hash,
                &block_header,
                is_finalized,
                spec_version,
                &trace,
                &mut tx,
            )
            .await?;
        // get extrinsic and event counts
        let extrinsic_count = util::get_extrinsic_count(&trace)?;
        let event_count = util::get_event_count(&trace)?;
        log::info!("Found {extrinsic_count} extrinsics and {event_count} events.");
        let events = self
            .get_events(&block_hash, spec_version, &metadata, &trace)
            .await?;
        self.process_events(
            &block_hash,
            &block_header,
            block_timestamp,
            spec_version,
            is_finalized,
            &events,
            &mut tx,
        )
        .await?;
        /*
        self.process_extrinsics(
            block_hash_hex,
            &block_header,
            block_timestamp,
            spec_version,
            metadata_version,
            &metadata,
            &trace,
            is_finalized,
            extrinsic_count,
            &mut tx,
        )
        .await?;
        */
        self.postgres
            .ingest_block(
                &block_hash,
                &block_header,
                block_timestamp,
                is_finalized,
                spec_version,
                extrinsic_count,
                event_count,
                &mut tx,
            )
            .await?;
        self.postgres
            .ingest_block_logs(&block_hash, &block_header, true, &mut tx)
            .await?;
        self.postgres.delete_error(&block_hash, &mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
}

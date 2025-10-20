#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::persistence::CrystalPostgreSQLStorage as _;
use crate::worker::{WorkerConfig, WorkerManager, WorkerType};
use anyhow::Context as _;
use async_trait::async_trait;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use submerge_base::types::substrate::chainspec::{ChainProperties, Chainspec};
use submerge_base::BaseService;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::RPCConfig;

mod api;
pub mod args;
mod metrics;
mod persistence;
mod types;
mod worker;

const RPC_URL: &str = "ws://104.247.178.13:5141";
// const RPC_URL: &str = "wss://public-rpc.mainnet.aventus.io";

pub struct Crystal {
    args: Args,
    postgres: Arc<PostgreSQLStorage>,
    worker_manager: Arc<WorkerManager>,
}

impl Crystal {
    pub async fn new(args: Args) -> anyhow::Result<Self> {
        let postgres = Arc::new(PostgreSQLStorage::new(&args.postgres).await?);
        Ok(Self {
            args,
            postgres,
            worker_manager: Default::default(),
        })
    }

    fn print_summary(&self, chainspec: &Chainspec) {
        log::info!(
            r#"
┌─────────────────────────────────────────────────────────────────────
│ Chain:    {}
└─────────────────────────────────────────────────────────────────────"#,
            chainspec.name,
        );
    }

    async fn launch_api(&self, chain_properties: ChainProperties) -> anyhow::Result<()> {
        api::run_api(
            chain_properties,
            &self.args.postgres,
            &self.worker_manager,
            &self.args.api.api_host,
            self.args.api.api_port,
        )
        .await
    }

    async fn process_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()> {
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
}

#[async_trait(?Send)]
impl BaseService for Crystal {
    fn get_name(&self) -> String {
        "💠 Submerge Crystal".to_string()
    }

    fn get_metrics_server_addr(&self) -> (String, u16) {
        (
            self.args.metrics.metrics_host.clone(),
            self.args.metrics.metrics_port,
        )
    }

    async fn run(&self) -> anyhow::Result<()> {
        let chainspec_json = fs::read_to_string(&self.args.chainspec_path)
            .context("🔴 Failed to read the chainspec file.")?;
        let chainspec: Chainspec =
            serde_json::from_str(&chainspec_json).context("🔴 Failed to parse chainspec JSON.")?;
        self.print_summary(&chainspec);
        self.process_genesis(&chainspec).await?;

        let recovery_duration = Duration::from_secs(self.args.service.recovery_sleep_seconds);

        self.worker_manager
            .spawn(
                WorkerType::SubscribeNewBlocks,
                WorkerConfig::new(
                    chainspec.properties.clone(),
                    self.postgres.clone(),
                    RPCConfig {
                        rpc_url: RPC_URL.to_string(),
                        rpc_connection_timeout_secs: 30,
                        rpc_request_timeout_secs: 30,
                        rpc_subscription_timeout_secs: 60,
                    },
                    self.args.legacy_decode_api_url.clone(),
                    recovery_duration,
                    false,
                    false,
                ),
            )
            .await;
        self.worker_manager
            .spawn(
                WorkerType::SubscribeFinalizedBlocks,
                WorkerConfig::new(
                    chainspec.properties.clone(),
                    self.postgres.clone(),
                    RPCConfig {
                        rpc_url: RPC_URL.to_string(),
                        rpc_connection_timeout_secs: 30,
                        rpc_request_timeout_secs: 30,
                        rpc_subscription_timeout_secs: 60,
                    },
                    self.args.legacy_decode_api_url.clone(),
                    recovery_duration,
                    false,
                    false,
                ),
            )
            .await;
        self.worker_manager
            .spawn(
                WorkerType::ProcessFinalizedRange {
                    maybe_start_block_number: Some(10_000_000),
                    maybe_end_block_number: Some(10_000_100),
                    scan: true,
                    reindex: false,
                },
                WorkerConfig::new(
                    chainspec.properties.clone(),
                    self.postgres.clone(),
                    RPCConfig {
                        rpc_url: RPC_URL.to_string(),
                        rpc_connection_timeout_secs: 30,
                        rpc_request_timeout_secs: 30,
                        rpc_subscription_timeout_secs: 60,
                    },
                    self.args.legacy_decode_api_url.clone(),
                    recovery_duration,
                    false,
                    true,
                ),
            )
            .await;
        self.launch_api(chainspec.properties.clone()).await?;
        Ok(())
    }
}

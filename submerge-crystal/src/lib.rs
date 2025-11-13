#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::persistence::CrystalPostgreSQLStorage as _;
use crate::worker::{WorkerConfig, WorkerManager, WorkerType};
use async_trait::async_trait;
use sqlx::migrate::Migrator;
use std::sync::Arc;
use std::time::Duration;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::BaseService;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::RPCConfig;

static DB_MIGRATOR: Migrator = sqlx::migrate!("../_migrations/crystal/migrations");

mod api;
pub mod args;
mod metrics;
mod persistence;
mod types;
mod worker;

//const RPC_URL: &str = "wss://acala.dotters.network";
//const RPC_URL: &str = "wss://astar-rpc.n.dwellir.com";
//const RPC_URL: &str = "wss://bifrost-polkadot.dotters.network";
//const RPC_URL: &str = "wss://rpc-centrifuge.luckyfriday.io";
//const RPC_URL: &str = "wss://hydration.dotters.network";
//const RPC_URL: &str = "wss://nexus.dotters.network";
//const RPC_URL: &str = "wss://moonbeam.dotters.network";
const RPC_URL: &str = "wss://mythos.dotters.network";
//const RPC_URL: &str = "wss://public-rpc.mainnet.aventus.io";
//const RPC_URL: &str = "wss://polkadot.dotters.network";
//const RPC_URL: &str = "wss://asset-hub-polkadot.dotters.network";

//const RPC_URL: &str = "wss://kusama.dotters.network";
//const RPC_URL: &str = "ws://104.247.178.13:5141";

//const RPC_URL: &str = "wss://bifrost-rpc.liebi.com/ws";
//const RPC_URL: &str = "wss:/moonriver.public.curie.radiumblock.co/ws";

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
        tracing::info!(
            r#"
┌─────────────────────────────────────────────────────────────────────
│ Chain:    {}
└─────────────────────────────────────────────────────────────────────"#,
            chainspec.name,
        );
    }

    async fn launch_api(&self, chain_name: &str) -> anyhow::Result<()> {
        api::run_api(
            chain_name.to_string(),
            &self.args.postgres,
            &self.worker_manager,
            &self.args.api.api_host,
            self.args.api.api_port,
        )
        .await
    }

    async fn process_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()> {
        tracing::info!("🔽 Processing genesis from chainspec file.");
        if self.postgres.get_genesis_record_count().await? > 0 {
            tracing::info!("🔁 Genesis had already been processed.");
            return Ok(());
        }
        self.postgres.ingest_genesis(chainspec).await?;
        tracing::info!(
            "✅ Processed {} storage items from the chainspec file.",
            chainspec.genesis.raw.top.len(),
        );
        Ok(())
    }

    async fn migrate_db(&self) -> anyhow::Result<()> {
        tracing::info!("🧰 Run database migrations.");
        DB_MIGRATOR.run(&self.postgres.connection_pool).await?;
        tracing::info!("✅ Database migrations completed.");
        Ok(())
    }
}

impl Crystal {
    async fn launch_dev_workers(&self, chain_name: &str) -> anyhow::Result<()> {
        let recovery_duration = Duration::from_secs(self.args.service.recovery_sleep_seconds);
        let worker_config = WorkerConfig {
            chain_name: chain_name.to_string(),
            postgres: self.postgres.clone(),
            rpc_config: RPCConfig {
                rpc_url: RPC_URL.to_string(),
                rpc_connection_timeout_secs: 30,
                rpc_request_timeout_secs: 30,
                rpc_subscription_timeout_secs: 60,
            },
            legacy_decode_api_url: self.args.legacy_decode_api_url.clone(),
            retry_delay: recovery_duration,
            skip_traces: true,
            stop_on_error: false,
        };
        self.worker_manager
            .spawn(WorkerType::SubscribeNewBlocks, worker_config.clone())
            .await;
        self.worker_manager
            .spawn(WorkerType::SubscribeFinalizedBlocks, worker_config.clone())
            .await;
        self.worker_manager
            .spawn(
                WorkerType::ProcessFinalizedRange {
                    maybe_start_block_number: Some(7_000_250),
                    maybe_end_block_number: Some(7_000_000),
                    scan: true,
                    reindex: false,
                },
                worker_config.clone(),
            )
            .await;
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

    fn get_external_log_level(&self) -> &str {
        &self.args.logging.external_log_level
    }

    fn get_native_log_level(&self) -> &str {
        &self.args.logging.native_log_level
    }

    async fn run(&self) -> anyhow::Result<()> {
        let chainspec = Chainspec::from_chain_name_or_file_path(&self.args.chain)?;
        self.print_summary(&chainspec);
        self.migrate_db().await?;
        self.process_genesis(&chainspec).await?;
        self.launch_dev_workers(&chainspec.name).await?;
        self.launch_api(&chainspec.name).await?;
        Ok(())
    }
}

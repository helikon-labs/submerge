#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::worker::{Worker, WorkerConfig, WorkerManager, WorkerType};
use async_trait::async_trait;

use sqlx::migrate::Migrator;
use std::sync::Arc;
use std::time::Duration;
use submerge_base::service::BaseService;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::RPCConfig;
use uuid::Uuid as UUID;

pub(crate) mod api;
pub use api::docs::APIDoc;
pub mod args;
mod metrics;
mod persistence;
mod types;
pub mod worker;

static DB_MIGRATOR: Migrator = sqlx::migrate!("../_migrations/crystal/migrations");

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
            &self.args.api,
        )
        .await
    }

    async fn process_genesis(
        &self,
        chainspec: &Chainspec,
        worker_config: &WorkerConfig,
    ) -> anyhow::Result<()> {
        let worker = Worker::new(
            chainspec.name.clone(),
            UUID::new_v4(),
            WorkerType::GenesisProcessor(chainspec.clone()),
            worker_config.clone(),
        );
        worker.start_failable().await
    }

    async fn migrate_db(&self) -> anyhow::Result<()> {
        tracing::info!("🧰 Run database migrations.");
        DB_MIGRATOR.run(&self.postgres.connection_pool).await?;
        tracing::info!("✅ Database migrations completed.");
        Ok(())
    }
}

impl Crystal {
    async fn launch_dev_workers(&self, worker_config: &WorkerConfig) -> anyhow::Result<()> {
        self.worker_manager
            .spawn(WorkerType::SubscribeNewBlocks, worker_config.clone())
            .await;
        self.worker_manager
            .spawn(WorkerType::SubscribeFinalizedBlocks, worker_config.clone())
            .await;
        self.worker_manager
            .spawn(
                WorkerType::ProcessFinalizedRange {
                    maybe_start_block_number: None,
                    maybe_end_block_number: None,
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
        let worker_config = WorkerConfig {
            chain_name: chainspec.name.clone(),
            postgres: self.postgres.clone(),
            rpc_config: RPCConfig {
                rpc_url: self.args.default_rpc_url.clone(),
                rpc_connection_timeout_secs: 30,
                rpc_request_timeout_secs: 30,
                rpc_subscription_timeout_secs: 60,
            },
            legacy_decode_api_url: self.args.legacy_decode_api_url.clone(),
            retry_delay: Duration::from_secs(self.args.service.recovery_sleep_seconds),
            skip_traces: true,
            stop_on_error: false,
        };
        self.print_summary(&chainspec);
        self.migrate_db().await?;
        self.process_genesis(&chainspec, &worker_config).await?;
        self.launch_dev_workers(&worker_config).await?;
        self.launch_api(&chainspec.name).await?;
        Ok(())
    }
}

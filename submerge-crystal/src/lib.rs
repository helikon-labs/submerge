#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::persistence::PostgreSQLStorage;
use async_trait::async_trait;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use submerge_base::args::{PostgreSQLArgs, RPCArgs};
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::BaseService;
use submerge_substrate_client::SubstrateClient;

mod api;
pub mod args;
mod metrics;
mod persistence;

lazy_static! {
    static ref IS_BUSY: AtomicBool = AtomicBool::new(false);
}

async fn get_postgres(args: &PostgreSQLArgs) -> anyhow::Result<PostgreSQLStorage> {
    PostgreSQLStorage::new(
        &args.postgres_host,
        args.postgres_port,
        &args.postgres_username,
        &args.postgres_password,
        &args.postgres_db_name,
        args.postgres_connection_timeout_secs,
        args.postgres_pool_max_connections,
    )
    .await
}

async fn get_substrate(args: &RPCArgs) -> anyhow::Result<SubstrateClient> {
    SubstrateClient::new(
        &args.http_rpc_url,
        &args.ws_rpc_url,
        args.rpc_connection_timeout_secs,
        args.rpc_request_timeout_secs,
    )
    .await
}

pub struct Crystal {
    args: Args,
}

impl Crystal {
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    async fn ingest_blocks(
        postgres: &PostgreSQLStorage,
        substrate_client: &SubstrateClient,
        start_block_number: u64,
        end_block_number: u64,
    ) -> anyhow::Result<()> {
        log::info!("⚙️ Ingest blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            log::info!("🔧 Ingesting block {number}. Target {end_block_number}.");
            let hash = substrate_client.get_block_hash(number).await?;
            if postgres.block_trace_exists(&hash).await? {
                log::info!("🔁 Block {number} had already been ingested.");
                continue;
            }
            let last_runtime_upgrade = substrate_client
                .get_last_runtime_upgrade_info(&hash)
                .await?;
            match substrate_client.get_block_trace(&hash).await {
                Ok(trace) => {
                    postgres
                        .ingest_block_trace(number, true, last_runtime_upgrade.spec_version, &trace)
                        .await?;
                    postgres.delete_trace_error(&hash).await?;
                    log::info!(
                        "🔽 Ingested {} traces for block {number}.",
                        trace.events.len(),
                    );
                }
                Err(error) => {
                    log::error!("❌ Error while getting traces for block {number}: {error:?}");
                    postgres
                        .save_trace_error(&hash, number, &error.to_string())
                        .await?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl BaseService for Crystal {
    fn get_metrics_server_addr(&self) -> Option<(String, u16)> {
        if self.args.no_metrics {
            None
        } else {
            Some((
                self.args.metrics.metrics_host.clone(),
                self.args.metrics.metrics_port,
            ))
        }
    }

    fn get_sleep_secs(&self) -> u64 {
        self.args.service.recovery_sleep_seconds
    }

    fn get_name(&self) -> String {
        "💠 Submerge Crystal".to_string()
    }

    async fn run(&self) -> anyhow::Result<()> {
        let chainspec_json = fs::read_to_string(&self.args.chainspec_path)?;
        let chainspec: Chainspec = serde_json::from_str(&chainspec_json)?;
        println!(
            r#"┌──────────────────────────────────────────────────────────────────────────────────────────
| Chain:            {}
│ HTTP RPC URL:     {}
│ WS RPC URL:       {}
│ Start Block:      {}
│ End Block:        {}
| API Enabled:      {}
| Metrics Enabled:  {}
└──────────────────────────────────────────────────────────────────────────────────────────"#,
            chainspec.name,
            self.args.rpc.http_rpc_url,
            self.args.rpc.ws_rpc_url,
            self.args
                .start_block
                .map_or("None".to_string(), |v| v.to_string()),
            self.args
                .end_block
                .map_or("None".to_string(), |v| v.to_string()),
            !self.args.no_api,
            !self.args.no_metrics,
        );

        if !self.args.no_api {
            let host = self.args.api.api_host.clone();
            let port = self.args.api.api_port;
            let postgres_args = self.args.postgres.clone();
            tokio::spawn(async move {
                let _ = api::run_api(&postgres_args, host.as_str(), port).await;
            });
        } else {
            log::info!("⛔ API disabled.");
        }

        match self.args.end_block {
            Some(end_block) => {
                let postgres = get_postgres(&self.args.postgres).await?;
                postgres.ingest_genesis(&chainspec).await?;
                let substrate_client = get_substrate(&self.args.rpc).await?;
                let start_block = self.args.start_block.unwrap_or(1);
                let next_block = if self.args.scan {
                    start_block
                } else {
                    postgres
                        .get_next_block_number(start_block, end_block)
                        .await?
                };
                if next_block < end_block {
                    Self::ingest_blocks(&postgres, &substrate_client, next_block, end_block)
                        .await?;
                } else {
                    log::info!("All blocks in range {start_block}-{end_block} had been ingested.");
                }
                Ok(())
            }
            None => loop {
                let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
                let postgres = Arc::new(get_postgres(&self.args.postgres).await?);
                postgres.ingest_genesis(&chainspec).await?;
                let substrate_client = Arc::new(get_substrate(&self.args.rpc).await?);
                substrate_client
                    .subscribe_to_finalized_blocks(
                        self.args.rpc.rpc_request_timeout_secs,
                        |finalized_block_header| {
                            let error_cell = error_cell.clone();
                            let postgres = postgres.clone();
                            let substrate_client = substrate_client.clone();
                            async move {
                                if let Some(error) = error_cell.get() {
                                    return Err(anyhow::anyhow!("{:?}", error));
                                }
                                let finalized_block_number = finalized_block_header.get_number()?;
                                log::info!("📦 New finalized block {finalized_block_number}.");

                                if IS_BUSY.load(Ordering::SeqCst) {
                                    log::info!("⏳ Busy ingesting past blocks. Skip block #{finalized_block_number}.");
                                    return Ok(());
                                }
                                IS_BUSY.store(true, Ordering::SeqCst);

                                let start_block = if self.args.scan {
                                    self.args.start_block.unwrap_or(1)
                                } else {
                                    postgres
                                        .get_next_block_number(self.args.start_block.unwrap_or(1), finalized_block_number)
                                        .await?
                                };
                                if start_block <= finalized_block_number {
                                    let postgres = postgres.clone();
                                    let substrate_client = substrate_client.clone();
                                    tokio::spawn(async move {
                                        if let Err(error) = Self::ingest_blocks(
                                            &postgres,
                                            &substrate_client,
                                            start_block,
                                            finalized_block_number,
                                        )
                                        .await
                                        {
                                            let _ = error_cell.set(error);
                                        }
                                        IS_BUSY.store(false, Ordering::SeqCst);
                                    });
                                } else {
                                    log::info!("🔁 Block {finalized_block_number} had already been ingested.");
                                    IS_BUSY.store(false, Ordering::SeqCst);
                                }
                                Ok(())
                            }
                        },
                    )
                    .await;
                let delay_seconds = self.args.service.recovery_sleep_seconds;
                log::error!("New block subscription exited. Will refresh connection and subscription after {delay_seconds} seconds.");
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}

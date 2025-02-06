#![warn(clippy::disallowed_types)]
use crate::args::Args;
use crate::persistence::PostgreSQLStorage;
use async_trait::async_trait;
use clap::Parser;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use submerge_base::BaseService;
use submerge_substrate_client::SubstrateClient;

mod api;
mod args;
mod metrics;
mod persistence;

lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref IS_BUSY: AtomicBool = AtomicBool::new(false);
}

pub struct Crystal {
    postgres: Arc<PostgreSQLStorage>,
    substrate_client: Arc<SubstrateClient>,
}

impl Crystal {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Crystal {
            postgres: Arc::new(
                PostgreSQLStorage::new(
                    &ARGS.postgres.postgres_host,
                    ARGS.postgres.postgres_port,
                    &ARGS.postgres.postgres_username,
                    &ARGS.postgres.postgres_password,
                    &ARGS.postgres.postgres_db_name,
                    ARGS.postgres.postgres_connection_timeout_secs,
                    ARGS.postgres.postgres_pool_max_connections,
                )
                .await?,
            ),
            substrate_client: Arc::new(
                SubstrateClient::new(
                    &ARGS.rpc.rpc_url,
                    ARGS.rpc.rpc_connection_timeout_secs,
                    ARGS.rpc.rpc_request_timeout_secs,
                )
                .await?,
            ),
        })
    }
}

impl Crystal {
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
            match substrate_client.get_block_trace(&hash).await {
                Ok(trace) => {
                    postgres.ingest_block_trace(number, true, &trace).await?;
                    log::info!(
                        "🔽 Ingested {} traces for block {number}.",
                        trace.events.len(),
                    );
                }
                Err(error) => {
                    log::error!(
                        "❌ Error while getting traces for block {number}: {:?}",
                        error,
                    );
                    postgres
                        .save_trace_error(&hash, number, &error.to_string())
                        .await?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl BaseService for Crystal {
    fn get_metrics_server_addr() -> Option<(&'static str, u16)> {
        if ARGS.no_metrics {
            None
        } else {
            Some((
                ARGS.metrics.metrics_host.as_str(),
                ARGS.metrics.metrics_port,
            ))
        }
    }

    fn get_sleep_secs() -> u64 {
        ARGS.service.recovery_sleep_seconds
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        let args = Args::parse();
        println!(
            r#"┌──────────────────────────────────────────────────────────────────────────────────────────
│ RPC URL:          {}
│ Start Block:      {}
│ End Block:        {}
│ Chainspec Path:   {}
| API Disabled:     {}
| Metrics Disabled: {}
└──────────────────────────────────────────────────────────────────────────────────────────"#,
            args.rpc.rpc_url,
            args.start_block
                .map_or("None".to_string(), |v| v.to_string()),
            args.end_block.map_or("None".to_string(), |v| v.to_string()),
            args.chainspec_path,
            args.no_api,
            args.no_metrics,
        );

        if !ARGS.no_api {
            tokio::spawn(async move {
                let _ = api::run_api(&args.api.api_host, args.api.api_port).await;
            });
        } else {
            log::info!("⛔ API disabled.");
        }

        self.postgres.ingest_genesis(&args.chainspec_path).await?;

        match ARGS.end_block {
            Some(end_block) => {
                let start_block = ARGS.start_block.unwrap_or(1);
                let next_block = self
                    .postgres
                    .get_next_block_number(start_block, end_block)
                    .await?;
                if next_block < end_block {
                    Self::ingest_blocks(
                        &self.postgres,
                        &self.substrate_client,
                        next_block,
                        end_block,
                    )
                    .await?;
                } else {
                    log::info!("All blocks in range {start_block}-{end_block} had been ingested.");
                }
                Ok(())
            }
            None => loop {
                let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
                IS_BUSY.store(false, Ordering::SeqCst);
                self.substrate_client
                    .subscribe_to_finalized_blocks(
                        ARGS.rpc.rpc_request_timeout_secs,
                        |finalized_block_header| {
                            let error_cell = error_cell.clone();
                            let postgres = self.postgres.clone();
                            let substrate_client = self.substrate_client.clone();
                            async move {
                                if let Some(error) = error_cell.get() {
                                    return Err(anyhow::anyhow!("{:?}", error));
                                }
                                let finalized_block_number = finalized_block_header.get_number()?;
                                log::info!("📦 New finalized block {finalized_block_number}.");

                                if IS_BUSY.load(Ordering::SeqCst) {
                                    log::info!(
                                        "⏳ Busy ingesting past blocks. Skip block #{}.",
                                        finalized_block_number
                                    );
                                    return Ok(());
                                }
                                IS_BUSY.store(true, Ordering::SeqCst);

                                let start_block = postgres
                                    .get_next_block_number(ARGS.start_block.unwrap_or(1), finalized_block_number)
                                    .await?;
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
                let delay_seconds = ARGS.service.recovery_sleep_seconds;
                log::error!(
                    "New block subscription exited. Will refresh connection and subscription after {} seconds.",
                    delay_seconds
                );
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}

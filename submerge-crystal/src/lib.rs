#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::process::BlockProcessor;
use async_trait::async_trait;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::BaseService;
use submerge_substrate_client::SubstrateClient;
use tokio::time::sleep;

mod api;
pub mod args;
mod metrics;
mod persistence;
mod process;
mod types;

lazy_static! {
    static ref IS_BUSY: AtomicBool = AtomicBool::new(false);
}

pub struct Crystal {
    args: Args,
}

impl Crystal {
    pub fn new(args: Args) -> Self {
        Self { args }
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
        let chainspec_json = fs::read_to_string(&self.args.chainspec_path)?;
        let chainspec: Chainspec = serde_json::from_str(&chainspec_json)?;
        println!(
            r#"┌──────────────────────────────────────────────────────────────────────────────────────────
| Chain:            {}
│ RPC URL:          {}
│ Start Block:      {}
│ End Block:        {}
| API Enabled:      {}
| Metrics Enabled:  {}
└──────────────────────────────────────────────────────────────────────────────────────────"#,
            chainspec.name,
            self.args.rpc.rpc_url,
            self.args
                .start_block
                .map_or("N/A".to_string(), |v| v.to_string()),
            self.args
                .end_block
                .map_or("N/A".to_string(), |v| v.to_string()),
            !self.args.no_api,
            !self.args.no_metrics,
        );

        // launch the API
        if !self.args.no_api {
            let host = self.args.api.api_host.clone();
            let port = self.args.api.api_port;
            let postgres_args = self.args.postgres.clone();
            tokio::spawn(async move {
                let _ = api::run_api(&postgres_args, host.as_str(), port).await;
            });
        } else {
            log::info!("ℹ️ API disabled.");
        }

        let block_processor = Arc::new(
            BlockProcessor::new(
                &self.args.postgres,
                &self.args.rpc,
                &self.args.legacy_decode_api_url,
            )
            .await?,
        );
        block_processor.process_genesis(&chainspec).await?;
        match self.args.end_block {
            Some(end_block) => {
                let start_block = self.args.start_block.unwrap_or(0);
                block_processor
                    .process_finalized_blocks_in_range(
                        self.args.scan,
                        self.args.stop_on_error,
                        self.args.skip_traces,
                        start_block,
                        end_block,
                    )
                    .await?;
                Ok(())
            }
            None => loop {
                let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
                let delay_seconds = self.args.service.recovery_sleep_seconds;
                let rpc_args = self.args.rpc.clone();
                let new_block_processor = block_processor.clone();
                let skip_traces = self.args.skip_traces;
                let new_block_subscription = tokio::spawn(async move {
                    loop {
                        let substrate_client = match SubstrateClient::new(&rpc_args).await {
                            Ok(substrate_client) => substrate_client,
                            Err(error) => {
                                log::error!("🔴 Error while contructing the Substrate client for new block subscription: {error:?}");
                                log::error!("🔄 Will retry after {delay_seconds} seconds.");
                                sleep(Duration::from_secs(delay_seconds)).await;
                                continue;
                            }
                        };
                        substrate_client
                            .subscribe_to_new_blocks(rpc_args.rpc_request_timeout_secs, |header| {
                                let block_processor = new_block_processor.clone();
                                async move {
                                    let hash_bytes = header.get_hash_bytes()?;
                                    let hash_hex = hex::encode(hash_bytes);
                                    let number = header.get_number()?;
                                    log::info!("🟦  New proposed block {number}.");
                                    block_processor
                                        .process_block(
                                            skip_traces,
                                            &hash_hex,
                                            number,
                                            types::BlockStatus::Proposed,
                                        )
                                        .await?;
                                    Ok(())
                                }
                            })
                            .await;
                        log::error!("🔴 New block subscription exited.");
                        log::error!("🔄 Will resubscribe after {delay_seconds} seconds.");
                        sleep(Duration::from_secs(delay_seconds)).await;
                    }
                });

                let rpc_args = self.args.rpc.clone();
                let start_block = self.args.start_block.unwrap_or(0);
                let scan = self.args.scan;
                let skip_traces = self.args.skip_traces;
                let stop_on_error = self.args.stop_on_error;
                let finalized_block_processor = block_processor.clone();
                let finalized_block_subscription = tokio::spawn(async move {
                    loop {
                        let substrate_client = match SubstrateClient::new(&rpc_args).await {
                            Ok(substrate_client) => substrate_client,
                            Err(error) => {
                                log::error!("🔴 Error while contructing the Substrate client for finalized block subscription: {error:?}");
                                log::error!("🔄 Will retry after {delay_seconds} seconds.");
                                sleep(Duration::from_secs(delay_seconds)).await;
                                continue;
                            }
                        };
                        substrate_client
                            .subscribe_to_finalized_blocks(
                                rpc_args.rpc_request_timeout_secs,
                                |header| {
                                    let error_cell = error_cell.clone();
                                    let block_processor = finalized_block_processor.clone();
                                    async move {
                                        if let Some(error) = error_cell.get() {
                                            return Err(anyhow::anyhow!("{:?}", error));
                                        }
                                        let start_block = start_block;
                                        let finalized_block_number = header.get_number()?;
                                        log::info!("🟩 New finalized block {finalized_block_number}.");
                                        if IS_BUSY.load(Ordering::SeqCst) {
                                            log::info!("⏳ Busy processing past finalized blocks. Skip finalized block {finalized_block_number}.");
                                            return Ok(());
                                        }
                                        IS_BUSY.store(true, Ordering::SeqCst);
                                        tokio::spawn(async move {
                                            if let Err(error) = block_processor
                                                .process_finalized_blocks_in_range(
                                                    scan,
                                                    stop_on_error,
                                                    skip_traces,
                                                    start_block,
                                                    finalized_block_number,
                                                )
                                                .await {
                                                let _ = error_cell.set(error);
                                            }
                                            IS_BUSY.store(false, Ordering::SeqCst);
                                        });
                                        Ok(())
                                    }
                                },
                            )
                            .await;
                        log::error!("🔴 Finalized block subscription exited.");
                        log::error!("🔄 Will resubscribe after {delay_seconds} seconds.");
                        sleep(Duration::from_secs(delay_seconds)).await;
                    }
                });

                let _ = tokio::join!(new_block_subscription, finalized_block_subscription);
                log::error!("🔴 All subscriptions exited.");
                log::error!(
                    "🔄 Will refresh connections and subscriptions after {delay_seconds} seconds."
                );
                sleep(Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}

#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::process::BlockProcessor;
use async_trait::async_trait;
use std::fs;
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
            log::info!("⛔ API disabled.");
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
                let delay_seconds = self.args.service.recovery_sleep_seconds;
                let rpc_args = self.args.rpc.clone();
                let new_block_processor = block_processor.clone();
                let skip_traces = self.args.skip_traces;
                let new_block_subscription = tokio::spawn(async move {
                    loop {
                        let substrate_client = match SubstrateClient::new(&rpc_args).await {
                            Ok(substrate_client) => substrate_client,
                            Err(error) => {
                                log::error!("Error while contructing the Substrate client for new block subscription: {error:?}");
                                log::error!("Will retry after {delay_seconds} seconds.");
                                sleep(Duration::from_secs(delay_seconds)).await;
                                continue;
                            }
                        };
                        substrate_client
                            .subscribe_to_new_blocks(rpc_args.rpc_request_timeout_secs, |header| {
                                let block_processor = new_block_processor.clone();
                                async move {
                                    let hash_bytes = header.get_hash_bytes().unwrap();
                                    let hash_hex = hex::encode(hash_bytes);
                                    let number = header.get_number().unwrap();
                                    log::info!("New block: {number} :: 0x{hash_hex}");
                                    block_processor
                                        .process_block(
                                            skip_traces,
                                            &hash_hex,
                                            number,
                                            types::BlockStatus::Proposed,
                                        )
                                        .await
                                        .unwrap();
                                    Ok(())
                                }
                            })
                            .await;
                        log::error!("New block subscription exited. Will refresh connection and subscription after {delay_seconds} seconds.");
                        sleep(Duration::from_secs(delay_seconds)).await;
                    }
                });

                let rpc_args = self.args.rpc.clone();
                let skip_traces = self.args.skip_traces;
                let finalized_block_processor = block_processor.clone();
                let finalized_block_subscription = tokio::spawn(async move {
                    loop {
                        let substrate_client = match SubstrateClient::new(&rpc_args).await {
                            Ok(substrate_client) => substrate_client,
                            Err(error) => {
                                log::error!("Error while contructing the Substrate client for finalized block subscription: {error:?}");
                                log::error!("Will retry after {delay_seconds} seconds.");
                                sleep(Duration::from_secs(delay_seconds)).await;
                                continue;
                            }
                        };
                        substrate_client
                            .subscribe_to_finalized_blocks(
                                rpc_args.rpc_request_timeout_secs,
                                |header| {
                                    let block_processor = finalized_block_processor.clone();
                                    async move {
                                        let mut number = header.get_number().unwrap();
                                        let hash_bytes = header.get_hash_bytes().unwrap();
                                        let mut hash_hex = hex::encode(hash_bytes);
                                        for _ in 0..10 {
                                            log::info!("Finalized block: {number} 0x{hash_hex}");
                                            // if nexists :: process
                                            block_processor
                                                .process_block(
                                                    skip_traces,
                                                    &hash_hex,
                                                    number,
                                                    types::BlockStatus::Finalized,
                                                )
                                                .await
                                                .unwrap();
                                            // if exists & not finalized :: finalize
                                            let parent_header = block_processor
                                                .get_parent_header(&hash_hex)
                                                .await
                                                .unwrap();
                                            hash_hex = parent_header.parent_hash.clone();
                                            number = parent_header.get_number().unwrap();
                                        }
                                        Ok(())
                                    }
                                },
                            )
                            .await;
                        log::error!("Finalized block subscription exited. Will re-subscribe after {delay_seconds} seconds.");
                        sleep(Duration::from_secs(delay_seconds)).await;
                    }
                });

                let _ = tokio::join!(new_block_subscription, finalized_block_subscription);
                log::error!("Subscriptions exited. Will refresh connections and subscriptions after {delay_seconds} seconds.");
                sleep(Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}

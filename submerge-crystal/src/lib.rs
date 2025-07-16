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
                    .process_blocks(
                        self.args.scan,
                        self.args.stop_on_error,
                        start_block,
                        end_block,
                    )
                    .await?;
                Ok(())
            }
            None => loop {
                let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
                let substrate_client = Arc::new(SubstrateClient::new(&self.args.rpc).await?);
                substrate_client
                    .subscribe_to_finalized_blocks(
                        self.args.rpc.rpc_request_timeout_secs,
                        |finalized_block_header| {
                            let error_cell = error_cell.clone();
                            let block_processor = block_processor.clone();
                            async move {
                                if let Some(error) = error_cell.get() {
                                    return Err(anyhow::anyhow!("{:?}", error));
                                }
                                let finalized_block_number = finalized_block_header.get_number()?;
                                log::info!("📦 New finalized block {finalized_block_number}.");

                                if IS_BUSY.load(Ordering::SeqCst) {
                                    log::info!("⏳ Busy processing past blocks. Skip block #{finalized_block_number}.");
                                    return Ok(());
                                }
                                IS_BUSY.store(true, Ordering::SeqCst);

                                let start_block_number = self.args.start_block.unwrap_or(0);
                                let scan = self.args.scan;
                                let stop_on_error = self.args.stop_on_error;
                                if start_block_number <= finalized_block_number {
                                    tokio::spawn(async move {
                                        if let Err(error) = block_processor.process_blocks(
                                            scan,
                                            stop_on_error,
                                            start_block_number,
                                            finalized_block_number,
                                        )
                                        .await
                                        {
                                            let _ = error_cell.set(error);
                                        }
                                        IS_BUSY.store(false, Ordering::SeqCst);
                                    });
                                } else {
                                    log::info!("🔁 Block {finalized_block_number} had already been processed.");
                                    IS_BUSY.store(false, Ordering::SeqCst);
                                }
                                Ok(())
                            }
                        },
                    )
                    .await;
                let delay_seconds = self.args.service.recovery_sleep_seconds;
                log::error!("New block subscription exited. Will refresh connection and subscription after {delay_seconds} seconds.");
                sleep(Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}

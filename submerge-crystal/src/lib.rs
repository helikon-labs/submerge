#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::process::BlockProcessor;
use anyhow::Context as _;
use async_trait::async_trait;
use std::fs;
use std::sync::Arc;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::BaseService;

mod api;
pub mod args;
mod metrics;
mod persistence;
mod process;
mod subscription;
mod types;

pub struct Crystal {
    args: Args,
}

impl Crystal {
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    fn print_summary(&self, chainspec: &Chainspec) {
        log::info!(
            r#"┌──────────────────────────────────────────────────────────────────────────────────────────
│ Chain:            {}
│ RPC URL:          {}
│ Start Block:      {}
│ End Block:        {}
│ API Enabled:      {}
│ Metrics Enabled:  {}
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
    }

    async fn launch_api(&self) {
        let host = self.args.api.api_host.clone();
        let port = self.args.api.api_port;
        let postgres_args = self.args.postgres.clone();
        tokio::spawn(async move {
            if let Err(error) = api::run_api(&postgres_args, &host, port).await {
                log::error!("❌ API failed: {error:?}");
            }
        });
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
            .context("Failed to read chainspec file")?;
        let chainspec: Chainspec =
            serde_json::from_str(&chainspec_json).context("Failed to parse chainspec JSON")?;

        self.print_summary(&chainspec);

        // launch the API
        if !self.args.no_api {
            self.launch_api().await;
        } else {
            log::info!("ℹ️ API disabled.");
        }

        let processor = Arc::new(
            BlockProcessor::new(
                &self.args.postgres,
                &self.args.rpc,
                &self.args.legacy_decode_api_url,
            )
            .await?,
        );
        processor.process_genesis(&chainspec).await?;

        match self.args.end_block {
            Some(end_block) => {
                let start_block = self.args.start_block.unwrap_or(0);
                processor
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
            None => self.subscribe_to_live_blocks(processor).await,
        }
    }
}

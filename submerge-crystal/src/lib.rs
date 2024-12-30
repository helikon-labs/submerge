#![warn(clippy::disallowed_types)]
use std::time::Duration;

use async_trait::async_trait;
use lazy_static::lazy_static;
use submerge_base::BaseService;
use submerge_config::Config;
use submerge_persistence::postgres::new_postgres_connection_pool;
use submerge_substrate_client::SubstrateClient;

mod api;
mod metrics;
mod persistence;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct Crystal;

#[async_trait(?Send)]
impl BaseService for Crystal {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (CONFIG.metrics.host.as_str(), CONFIG.metrics.crystal_port)
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!(":: Submerge Crystal ::");
        let postgres = new_postgres_connection_pool(&CONFIG).await?;
        tokio::spawn(async move {
            let _ = api::run_api(&CONFIG.http.service_host, CONFIG.http.crystal_api_port).await;
        });

        let substrate_client =
            SubstrateClient::new("wss://rpc.helikon.io/polkadot", 30, 30).await?;
        for number in 24_000_000..24_001_000 {
            log::info!("Process block {number}.");
            let hash = substrate_client.get_block_hash(number as u64).await?;
            let trace = substrate_client.get_block_trace(&hash).await?;
            persistence::save_block_trace(&postgres, 10, &trace).await?;
            log::info!("Saved {} traces for block {number}.", trace.events.len());
        }
        loop {
            log::info!("Process block.");
            tokio::time::sleep(Duration::from_millis(2_000)).await;
        }
    }
}

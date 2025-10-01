use dv_report_config::Config;
use dv_report_persistence::postgres::PostgreSQLStorage;
use dv_report_subsquare_client::SubsquareClient;
use dv_report_substrate_client::SubstrateClient;

pub mod block;
pub mod cohort;
pub mod network;
pub mod referendum;

pub struct Repository {
    postgres: PostgreSQLStorage,
    subsquare_client: SubsquareClient,
    substrate_client: SubstrateClient,
}

impl Repository {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            postgres: PostgreSQLStorage::new(config).await?,
            subsquare_client: SubsquareClient::new(config)?,
            substrate_client: SubstrateClient::new(
                config.substrate.rpc_url.as_str(),
                config.substrate.connection_timeout_seconds,
                config.substrate.request_timeout_seconds,
                &config.indexer.metadata_file_path,
            )
            .await?,
        })
    }

    pub fn set_metadata(&self, path: &str) -> anyhow::Result<()> {
        self.substrate_client.set_metadata(path)
    }
}

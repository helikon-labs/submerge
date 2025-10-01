use crate::Repository;
use dv_report_types::substrate::network::Network;

impl Repository {
    pub async fn get_all_networks(&self) -> anyhow::Result<Vec<Network>> {
        self.postgres.get_all_networks().await
    }
}

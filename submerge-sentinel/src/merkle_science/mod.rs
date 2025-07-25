#![warn(clippy::disallowed_types)]

use crate::args::Args;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use types::{
    AddressScreening, AddressScreeningRequest, AggregateServiceStatusResponse, Blockchain,
    BlockchainListResponse, BlockchainServiceStatus, SupportedDigitalAsset,
    SupportedDigitalAssetListResponse, TransactionScreening, TransactionScreeningRequest,
};

pub mod types;

pub struct MerkleScienceClient {
    api_key: String,
    http_client: reqwest::Client,
    supported_blockchains: Vec<Blockchain>,
}

impl MerkleScienceClient {
    pub async fn new(args: &Args) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(args.request_timeout_secs))
            .build()?;
        let supported_blockchains =
            Self::get_supported_blockchains(&http_client, &args.merkle_science_api_key).await?;
        Ok(Self {
            api_key: args.merkle_science_api_key.clone(),
            http_client,
            supported_blockchains,
        })
    }

    async fn get_supported_blockchains(
        http_client: &reqwest::Client,
        api_key: &str,
    ) -> anyhow::Result<Vec<Blockchain>> {
        let response = http_client
            .get("https://api.merklescience.com/api/v4.2/blockchains/")
            .header("X-API-KEY", api_key)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message =
                format!("Error while fetching supported blockchains: {response_text}");
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: BlockchainListResponse = serde_json::from_str(&response_text)?;
        Ok(response.results)
    }

    pub fn get_blockchain_by_name(&self, name: &str) -> anyhow::Result<Blockchain> {
        match self
            .supported_blockchains
            .iter()
            .find(|b| b.name.to_lowercase() == name.to_lowercase())
        {
            Some(blockchain) => Ok(blockchain.clone()),
            None => Err(anyhow::Error::msg(format!(
                "{name} not found in supported blockchains."
            ))),
        }
    }

    pub async fn get_blockchain_supported_digital_assets(
        &self,
        blockchain: &Blockchain,
    ) -> anyhow::Result<Vec<SupportedDigitalAsset>> {
        let url = format!(
            "https://api.merklescience.com/api/v4.2/blockchains/{}/digital-assets/",
            blockchain.id
        );
        let response = self
            .http_client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message = format!(
                "Error while fetching supported digital assets for {}.",
                blockchain.name,
            );
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: SupportedDigitalAssetListResponse = serde_json::from_str(&response_text)?;
        Ok(response.results)
    }

    pub async fn get_blockchain_service_health(
        &self,
        blockchain: &Blockchain,
    ) -> anyhow::Result<BlockchainServiceStatus> {
        let response = self
            .http_client
            .get("https://api.merklescience.com/api/v4.2/health-check/")
            .header("X-API-KEY", &self.api_key)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message = "Error while fetching aggregate service status.";
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: AggregateServiceStatusResponse = serde_json::from_str(&response_text)?;
        match response
            .statuses_by_blockchain
            .iter()
            .find(|s| s.blockchain.id == blockchain.id)
        {
            Some(status) => Ok(status.clone()),
            None => Err(anyhow::Error::msg(format!(
                "Status not found for {}.",
                blockchain.name
            ))),
        }
    }

    pub async fn screen_address(
        &self,
        address: &str,
        blockchain: &Blockchain,
    ) -> anyhow::Result<AddressScreening> {
        let request = AddressScreeningRequest {
            identifier: address.to_string(),
            blockchain: blockchain.id,
            customer_id: None,
            transfer_type: None,
            show_alerts: None,
            custom_tags: None,
        };
        log::info!("Screening {} address: {address}", blockchain.name);
        let response_result = self
            .http_client
            .post("https://api.merklescience.com/api/v4.2/addresses/")
            .header("X-API-KEY", &self.api_key)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error while screening address: {error}");
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message = format!("Error response from the API: {response_text}");
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: AddressScreening = serde_json::from_str(&response_text)?;
        log::info!(
            "Screening completed for address {address} on {}.",
            blockchain.name,
        );
        Ok(response)
    }

    pub async fn screen_transaction(
        &self,
        identifier: &str,
        blockchain: &Blockchain,
    ) -> anyhow::Result<TransactionScreening> {
        let request = TransactionScreeningRequest {
            identifier: identifier.to_string(),
            blockchain: blockchain.id,
            customer_id: None,
            address: None,
            transfer_type: None,
            show_alerts: None,
        };
        log::info!("Screening {} transaction: {identifier}", blockchain.name);
        let response_result = self
            .http_client
            .post("https://api.merklescience.com/api/v4.2/transactions/")
            .header("X-API-KEY", &self.api_key)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error while screening address: {error}");
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message = format!("Error response from the API: {response_text}");
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: TransactionScreening = serde_json::from_str(&response_text)?;
        log::info!(
            "Screening completed for transaction {identifier} on {}.",
            blockchain.name,
        );
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::types::RiskLevel;
    use super::MerkleScienceClient;
    use crate::args::Args;
    use clap::Parser;

    #[test_log::test(tokio::test)]
    async fn test_screen_normal_address() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = MerkleScienceClient::new(&args).await?;
        let address = "15fTH34bbKGMUjF1bLmTqxPYgpg481imThwhWcQfCyktyBzL";
        let chain = client.get_blockchain_by_name("polkadot")?;
        let screening = client.screen_address(address, &chain).await?;
        assert_eq!(screening.blockchain.id, chain.id);
        assert_eq!(screening.identifier, address);
        assert_eq!(screening.risk.level, RiskLevel::NoRisk);
        assert_eq!(screening.tags.owner, None);
        assert_eq!(screening.tags.user, None);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_screen_exchange_address() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = MerkleScienceClient::new(&args).await?;
        let address = "155YSCfVaqwk8cEr46FGVw8rNW96oBnHq9A9SXoCsr2MGj9i";
        let chain = client.get_blockchain_by_name("polkadot")?;
        let screening = client.screen_address(address, &chain).await?;
        assert_eq!(screening.blockchain.id, chain.id);
        assert_eq!(screening.identifier, address);
        assert_eq!(screening.risk.level, RiskLevel::NoRisk);
        assert!(screening.tags.owner.is_some());
        assert_eq!(screening.tags.owner.unwrap().tag_type, "Exchange");
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_screen_transaction() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = MerkleScienceClient::new(&args).await?;
        let identifier = "0x7ff1606ccf39f31714ddffe35f73704998603bc7a8313703de80ee92b6dca468";
        let chain = client.get_blockchain_by_name("polkadot")?;
        let screening = client.screen_transaction(identifier, &chain).await?;
        assert_eq!(screening.blockchain.id, chain.id);
        assert_eq!(screening.identifier, identifier);
        assert_eq!(screening.risk.level, RiskLevel::NoRisk);
        assert_eq!(screening.value, "46.7969");
        assert_eq!(screening.fee, "0.0174");
        assert!(!screening.originators.is_empty());
        assert!(!screening.beneficiaries.is_empty());
        assert!(!screening.digital_assets.is_empty());
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_polkadot_supported_digital_assets() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = MerkleScienceClient::new(&args).await?;
        let chain = client.get_blockchain_by_name("polkadot")?;
        let assets = client
            .get_blockchain_supported_digital_assets(&chain)
            .await?;
        assert!(!assets.is_empty());
        assert_eq!(assets.first().unwrap().symbol, "DOT");
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_polkadot_service_health() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = MerkleScienceClient::new(&args).await?;
        let chain = client.get_blockchain_by_name("polkadot")?;
        let status = client.get_blockchain_service_health(&chain).await?;
        assert!(status.last_synced_block > 25202200);
        Ok(())
    }
}

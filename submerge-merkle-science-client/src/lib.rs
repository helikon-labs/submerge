#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::types::{AddressScreening, AddressScreeningRequest, Blockchain, BlockchainListResponse};
use reqwest::header::{ACCEPT, CONTENT_TYPE};

pub mod args;
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
            let error_message = format!(
                "Error while fetching supported blockchains: {}",
                response_text
            );
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
                "{} not found in supported blockchains.",
                name,
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
        log::info!("Sending address to {}", address);
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
                log::error!("Error while screening address: {}", error);
                return Err(error.into());
            }
        };
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message = format!("Error response from the API: {}", response_text);
            log::error!("{error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: AddressScreening = serde_json::from_str(&response_text)?;
        log::info!(
            "Screening completed for address {} on {}.",
            address,
            blockchain.name,
        );
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::types::RiskLevel;
    use crate::MerkleScienceClient;
    use clap::Parser;

    #[test_log::test(tokio::test)]
    async fn test_screen_normal_address() -> anyhow::Result<()> {
        let args = Args::parse();
        log::info!("API KEY: {}", args.merkle_science_api_key);
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
}

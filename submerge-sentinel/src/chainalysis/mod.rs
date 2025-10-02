use reqwest::header::ACCEPT;

use crate::{args::Args, chainalysis::types::SanctionStatus};

pub mod types;

pub struct ChainalysisClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl ChainalysisClient {
    pub async fn new(args: &Args) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(args.request_timeout_secs))
            .build()?;
        Ok(Self {
            api_key: args.chainalysis_api_key.clone(),
            http_client,
        })
    }

    pub async fn get_sanction_status(&self, address: &str) -> anyhow::Result<SanctionStatus> {
        let url = format!(" https://public.chainalysis.com/api/v1/address/{address}");
        let response = self
            .http_client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status_code = response.status();
        let response_text = response.text().await?;
        if !status_code.is_success() {
            let error_message =
                format!("Error while checking Chainalysis sanction status for address {address}.");
            log::error!("⚠️ {error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: SanctionStatus = serde_json::from_str(&response_text)?;
        Ok(response)
    }
}

#[cfg(all(test, feature = "integration-test"))]
mod tests {
    use super::ChainalysisClient;
    use crate::args::Args;
    use clap::Parser;

    #[test_log::test(tokio::test)]
    async fn test_screen_sanctioned_address_1() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = ChainalysisClient::new(&args).await?;
        let address = "0x1da5821544e25c636c1417ba96ade4cf6d2f9b5a";
        let status = client.get_sanction_status(address).await?;
        assert!(status.identifications.len() > 1);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_screen_sanctioned_address_2() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = ChainalysisClient::new(&args).await?;
        let address = "TDdbRFoBTEmE3qiR69Y6rKRSG1hoF65QaE";
        let status = client.get_sanction_status(address).await?;
        assert!(status.identifications.len() > 0);
        Ok(())
    }
}

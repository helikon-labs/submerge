use reqwest::header::ACCEPT;

use crate::{args::Args, scorechain::types::SanctionStatus};

pub mod types;

pub struct ScorechainClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl ScorechainClient {
    pub async fn new(args: &Args) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(args.request_timeout_secs))
            .build()?;
        Ok(Self {
            api_key: args.scorechain_api_key.clone(),
            http_client,
        })
    }

    pub async fn get_sanction_status(&self, address: &str) -> anyhow::Result<SanctionStatus> {
        let url = format!("https://sanctions.api.scorechain.com/v1/addresses/{address}");
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
                format!("Error while checking sanctioned status for address {address}.");
            log::error!("⚠️ {error_message}");
            return Err(anyhow::Error::msg(error_message));
        }
        let response: SanctionStatus = serde_json::from_str(&response_text)?;
        Ok(response)
    }
}

#[cfg(all(test, feature = "integration-test"))]
mod tests {
    use super::ScorechainClient;
    use crate::args::Args;
    use clap::Parser;

    #[test_log::test(tokio::test)]
    async fn test_screen_normal_address() -> anyhow::Result<()> {
        let args = Args::parse();
        let client = ScorechainClient::new(&args).await?;
        let address = "15fTH34bbKGMUjF1bLmTqxPYgpg481imThwhWcQfCyktyBzL";
        let status = client.get_sanction_status(address).await?;
        assert!(!status.is_sanctioned);
        Ok(())
    }
}

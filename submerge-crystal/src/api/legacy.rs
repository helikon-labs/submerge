use crate::types::legacy::LegacyEventWrapper;
use crate::types::legacy::LegacyExtrinsicWrapper;
use serde::Serialize;

pub struct LegacyDecodeAPIClient {
    url: String,
    http_client: reqwest::Client,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DecodeRequest {
    block_hash: String,
    spec_version: u32,
    hex: String,
}

impl LegacyDecodeAPIClient {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            url: url.to_owned(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()?,
        })
    }

    pub async fn decode_extrinsic(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> anyhow::Result<LegacyExtrinsicWrapper> {
        let url = format!("{}/decode/extrinsic", self.url);
        let request = DecodeRequest {
            block_hash: format!("0x{}", hex::encode(block_hash)),
            spec_version,
            hex: format!("0x{}", hex::encode(bytes)),
        };
        let response_result = self.http_client.post(url).json(&request).send().await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                return Err(error.into());
            }
        };
        let status_code = response.status();
        if !status_code.is_success() {
            let response_text = response.text().await?;
            return Err(anyhow::Error::msg(response_text));
        }
        let extrinsic_wrapper = response.json::<LegacyExtrinsicWrapper>().await?;
        Ok(extrinsic_wrapper)
    }

    pub async fn decode_event(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> anyhow::Result<LegacyEventWrapper> {
        let url = format!("{}/decode/event", self.url);
        let request = DecodeRequest {
            block_hash: format!("0x{}", hex::encode(block_hash)),
            spec_version,
            hex: format!("0x{}", hex::encode(bytes)),
        };
        let response_result = self.http_client.post(url).json(&request).send().await;
        let response = match response_result {
            Ok(response) => response,
            Err(error) => {
                return Err(error.into());
            }
        };
        let status_code = response.status();
        if !status_code.is_success() {
            let response_text = response.text().await?;
            return Err(anyhow::Error::msg(response_text));
        }
        let event_wrapper = response.json::<LegacyEventWrapper>().await?;
        Ok(event_wrapper)
    }
}

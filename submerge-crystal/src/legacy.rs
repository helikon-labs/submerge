use serde::Serialize;

pub struct LegacyDecodeAPIClient {
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
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
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
    ) -> anyhow::Result<String> {
        let url = "http://localhost:7070/decode/extrinsic";
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
        let response_text = response.text().await?;
        if !status_code.is_success() {
            return Err(anyhow::Error::msg(response_text));
        }
        Ok(response_text)
    }

    pub async fn decode_event(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> anyhow::Result<String> {
        let url = "http://localhost:7070/decode/event";
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
        let response_text = response.text().await?;
        if !status_code.is_success() {
            return Err(anyhow::Error::msg(response_text));
        }
        Ok(response_text)
    }
}

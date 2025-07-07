use serde::{Deserialize, Serialize};
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacyEventWrapper {
    pub phase: LegacyEventPhase,
    pub event: LegacyEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacyEventPhase {
    #[serde(rename = "type")]
    pub ty: String,
    pub value: JsonValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacyEvent {
    #[serde(rename = "method")]
    pub name: String,
    #[serde(rename = "section")]
    pub pallet: String,
    #[serde(rename = "index")]
    pub index_hex: String,
    pub data: JsonValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacyExtrinsicWrapper {
    pub is_signed: bool,
    #[serde(rename = "method")]
    pub call: LegacyCall,
    #[serde(flatten)]
    pub signature: Option<LegacySignature>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacyCall {
    #[serde(rename = "method")]
    pub name: String,
    #[serde(rename = "section")]
    pub pallet: String,
    pub args: JsonMap<String, JsonValue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacySignature {
    pub nonce: String,
    pub signature: String,
    pub signer: String,
    pub tip: String,
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
    ) -> anyhow::Result<LegacyExtrinsicWrapper> {
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
        if !status_code.is_success() {
            let response_text = response.text().await?;
            return Err(anyhow::Error::msg(response_text));
        }
        let event_wrapper = response.json::<LegacyEventWrapper>().await?;
        Ok(event_wrapper)
    }
}

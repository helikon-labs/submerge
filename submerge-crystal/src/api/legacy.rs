use crate::types::legacy::LegacyCall;
use crate::types::legacy::LegacyEventWrapper;
use crate::types::legacy::LegacyExtrinsicWrapper;
use crate::types::legacy::LegacyMultiaddress;
use crate::types::legacy::MultiaddressType;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JSONValue;
use std::time::Duration;

#[derive(Debug)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub _retries: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            _retries: 3,
        }
    }
}

pub struct LegacyDecodeAPIClient {
    url: String,
    http_client: reqwest::Client,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyExtrinsicWrapperIntermediate {
    pub is_signed: bool,
    #[serde(rename = "method")]
    pub call: LegacyCall,
    pub signature: Option<String>,
    pub signer: Option<JSONValue>,
    pub era: Option<JSONValue>,
    pub nonce: Option<String>,
    pub tip: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DecodeRequest {
    block_hash: String,
    spec_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_name: Option<String>,
    hex: String,
}

impl LegacyDecodeAPIClient {
    pub fn new(url: &str) -> Result<Self> {
        Self::new_with_config(url, ClientConfig::default())
    }

    pub fn new_with_config(url: &str, config: ClientConfig) -> Result<Self> {
        if url.is_empty() {
            anyhow::bail!("URL cannot be empty");
        }
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self {
            url: url.trim_end_matches('/').to_owned(),
            http_client,
        })
    }

    async fn make_decode_request<T>(&self, endpoint: &str, request: &DecodeRequest) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.url, endpoint);

        let response = self
            .http_client
            .post(&url)
            .json(request)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {url}."))?;
        let status = response.status();
        if !status.is_success() {
            let response_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response.".to_string());
            anyhow::bail!("API request failed with status {status}: {response_text}");
        }
        response
            .json::<T>()
            .await
            .context("Failed to deserialize response JSON")
    }

    fn create_decode_request(
        block_hash: &[u8],
        spec_version: u32,
        type_name: Option<String>,
        bytes: &[u8],
    ) -> DecodeRequest {
        DecodeRequest {
            block_hash: format!("0x{}", hex::encode(block_hash)),
            spec_version,
            type_name,
            hex: format!("0x{}", hex::encode(bytes)),
        }
    }

    fn process_signer(signer_value: &JSONValue) -> Result<LegacyMultiaddress> {
        match signer_value {
            JSONValue::String(account_id_hex) => Ok(LegacyMultiaddress {
                ty: MultiaddressType::Id,
                value: account_id_hex.clone(),
            }),
            JSONValue::Object(_) => serde_json::from_value(signer_value.clone())
                .context("Failed to deserialize signer object"),
            _ => anyhow::bail!(
                "Signer must be either a string or object, got: {:?}",
                signer_value
            ),
        }
    }

    pub async fn decode_extrinsic(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> Result<LegacyExtrinsicWrapper> {
        let request = Self::create_decode_request(block_hash, spec_version, None, bytes);
        let intermediate: LegacyExtrinsicWrapperIntermediate = self
            .make_decode_request("decode/extrinsic", &request)
            .await
            .context("Failed to decode extrinsic.")?;
        let signer = intermediate
            .signer
            .as_ref()
            .map(Self::process_signer)
            .transpose()
            .context("Failed to process signer field.")?;
        Ok(LegacyExtrinsicWrapper {
            is_signed: intermediate.is_signed,
            call: intermediate.call,
            signature: intermediate.signature,
            signer,
            era: intermediate.era,
            nonce: intermediate.nonce,
            tip: intermediate.tip,
        })
    }

    pub async fn decode_event(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> Result<LegacyEventWrapper> {
        let request = Self::create_decode_request(block_hash, spec_version, None, bytes);
        self.make_decode_request("decode/event", &request)
            .await
            .context("Failed to decode event")
    }

    pub async fn decode_events(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> Result<Vec<LegacyEventWrapper>> {
        let request = Self::create_decode_request(block_hash, spec_version, None, bytes);
        self.make_decode_request("decode/events", &request)
            .await
            .context("Failed to decode events")
    }

    pub async fn decode_block_weight(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> Result<JSONValue> {
        let request = Self::create_decode_request(block_hash, spec_version, None, bytes);
        self.make_decode_request("decode/block-weight", &request)
            .await
            .context("Failed to decode event")
    }

    pub async fn decode_type(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        type_name: &str,
        bytes: &[u8],
    ) -> Result<JSONValue> {
        let request = Self::create_decode_request(
            block_hash,
            spec_version,
            Some(type_name.to_owned()),
            bytes,
        );
        self.make_decode_request("decode/type", &request)
            .await
            .context("Failed to decode event")
    }
}

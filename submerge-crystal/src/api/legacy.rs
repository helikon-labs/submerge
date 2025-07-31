use crate::types::legacy::LegacyCall;
use crate::types::legacy::LegacyEventWrapper;
use crate::types::legacy::LegacyExtrinsicWrapper;
use crate::types::legacy::LegacyMultiaddress;
use crate::types::legacy::MultiaddressType;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

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
    pub signer: Option<JsonValue>,
    pub era: Option<JsonValue>,
    pub nonce: Option<String>,
    pub tip: Option<String>,
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
        let intermediate: LegacyExtrinsicWrapperIntermediate = response.json().await?;
        let mut wrapper = LegacyExtrinsicWrapper {
            is_signed: intermediate.is_signed,
            call: intermediate.call,
            signature: intermediate.signature,
            signer: None,
            era: intermediate.era,
            nonce: intermediate.nonce,
            tip: intermediate.tip,
        };
        if let Some(signer) = &intermediate.signer {
            let signer = match signer {
                JsonValue::String(account_id_hex) => LegacyMultiaddress {
                    ty: MultiaddressType::Id,
                    value: account_id_hex.clone(),
                },
                JsonValue::Object(_) => {
                    let signer_json = serde_json::to_string(signer)?;
                    serde_json::from_str::<LegacyMultiaddress>(&signer_json)?
                }
                _ => anyhow::bail!("Unexpected type for signer field in legacy extrinsic."),
            };
            wrapper.signer = Some(signer);
        }
        Ok(wrapper)
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

    pub async fn decode_events(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &[u8],
    ) -> anyhow::Result<Vec<LegacyEventWrapper>> {
        let url = format!("{}/decode/events", self.url);
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
        let event_wrappers = response.json::<Vec<LegacyEventWrapper>>().await?;
        Ok(event_wrappers)
    }
}

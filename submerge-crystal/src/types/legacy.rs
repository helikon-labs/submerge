use serde::{Deserialize, Serialize};
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacyEventWrapper {
    phase: JsonValue,
    pub event: LegacyEvent,
}

impl LegacyEventWrapper {
    pub fn get_phase(&self) -> anyhow::Result<LegacyEventPhase> {
        match &self.phase {
            JsonValue::String(ty) => Ok(LegacyEventPhase {
                ty: ty.clone(),
                value: JsonValue::Null,
            }),
            JsonValue::Object(_) => {
                let json_str = serde_json::to_string(&self.phase)?;
                Ok(serde_json::from_str(&json_str)?)
            }
            _ => unimplemented!("Unexpected event phase type."),
        }
    }
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

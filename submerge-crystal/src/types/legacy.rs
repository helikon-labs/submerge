use serde::{Deserialize, Serialize};
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use sp_runtime::AccountId32;
use submerge_base::types::substrate::MultiAddress;

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
    pub signature: Option<String>,
    pub signer: Option<LegacyMultiaddress>,
    pub era: Option<JsonValue>,
    pub nonce: Option<String>,
    pub tip: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyCall {
    #[serde(rename = "method")]
    pub pallet_call_name: String,
    #[serde(rename = "section")]
    pub pallet_name: String,
    pub args: JsonMap<String, JsonValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MultiaddressType {
    Id,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyMultiaddress {
    #[serde(rename = "type")]
    pub ty: MultiaddressType,
    pub value: String,
}

impl TryFrom<&LegacyMultiaddress> for MultiAddress {
    type Error = anyhow::Error;

    fn try_from(value: &LegacyMultiaddress) -> Result<Self, Self::Error> {
        match value.ty {
            MultiaddressType::Id => {
                let bytes = hex::decode(value.value.trim_start_matches("0x"))?;
                let bytes: [u8; 32] = bytes.try_into().expect("Cannot convert account id.");
                Ok(MultiAddress::Id(AccountId32::new(bytes)))
            }
        }
    }
}

use serde::Serialize;
use serde_json::Value as JSONValue;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub spec_version: u32,
    pub metadata_version: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPallet {
    pub index: u32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPalletCall {
    pub index: u32,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPalletConstant {
    pub index: u32,
    pub name: String,
    pub type_id: Option<u32>,
    pub type_name: String,
    pub value_hex: String,
    pub value: Option<JSONValue>,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPalletError {
    pub index: u32,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPalletEvent {
    pub index: u32,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPalletStorageItem {
    pub index: u32,
    pub name: String,
    pub key: String,
    pub docs: Vec<String>,
}

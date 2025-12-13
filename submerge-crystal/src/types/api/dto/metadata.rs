use serde::Serialize;
use serde_json::Value as JSONValue;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataFullDTO {
    pub spec_version: u32,
    pub metadata_version: u32,
    pub pallets: Vec<MetadataPalletFullDTO>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPalletDTO {
    pub index: u32,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataPalletFullDTO {
    pub index: u32,
    pub name: String,
    pub calls: Vec<MetadataCallDTO>,
    pub constants: Vec<MetadataConstantDTO>,
    pub errors: Vec<MetadataErrorDTO>,
    pub events: Vec<MetadataEventDTO>,
    pub storage_items: Vec<MetadataStorageItemDTO>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataCallDTO {
    pub index: u32,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataConstantDTO {
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
pub struct MetadataErrorDTO {
    pub index: u32,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataEventDTO {
    pub index: u32,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStorageItemDTO {
    pub index: u32,
    pub name: String,
    pub key_prefix: String,
    pub docs: Vec<String>,
}

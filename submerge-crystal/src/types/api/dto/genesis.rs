use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisRecordDTO {
    pub key_prefix: String,
    pub key_params: Option<String>,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_storage_item_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_storage_item_name: Option<String>,
}

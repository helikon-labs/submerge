use serde::Serialize;

use crate::types::api::dto::trace::TraceType;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisRecordDTO {
    pub key_prefix: String,
    pub key_params: Option<String>,
    pub value: String,
    pub record_type: Option<TraceType>,
    pub pallet_index: Option<u32>,
    pub pallet_name: Option<String>,
    pub pallet_storage_item_index: Option<u32>,
    pub pallet_storage_item_name: Option<String>,
}

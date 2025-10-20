use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisRecordDTO {
    pub id: u64,
    pub key: String,
    pub key_prefix: String,
    pub value: String,
}

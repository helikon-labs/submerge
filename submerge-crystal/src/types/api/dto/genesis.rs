use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisRecord {
    pub id: u64,
    pub key: String,
    pub value: String,
}

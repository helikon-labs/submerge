use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionIdentification {
    pub category: String,
    pub name: String,
    pub description: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionStatus {
    pub identifications: Vec<SanctionIdentification>,
}

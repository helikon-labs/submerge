use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionStatus {
    pub is_sanctioned: bool,
    pub details: Option<String>,
}

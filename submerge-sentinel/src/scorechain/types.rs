use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionDetails {
    pub name: String,
    #[serde(rename = "sanctionDate")]
    pub sanction_timestamp: u64,
    #[serde(rename = "prettySanctionDate")]
    pub sanction_date: DateTime<Utc>,
    pub blockchain: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionStatus {
    pub is_sanctioned: bool,
    pub details: Option<SanctionDetails>,
}

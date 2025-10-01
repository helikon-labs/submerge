use crate::substrate::block::Block;
use crate::substrate::network::Network;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cohort {
    pub number: u32,
    pub network: Network,
    pub announcement_date: NaiveDateTime,
    pub announcement_url: Option<String>,
    pub delegation_date: NaiveDateTime,
    pub start_block: Block,
}

#[derive(Clone, Debug, FromRow)]
pub struct CohortRow {
    pub number: i32,
    pub network_id: i32,
    pub announcement_date: NaiveDateTime,
    pub announcement_url: Option<String>,
    pub delegation_date: NaiveDateTime,
    pub start_block_hash: String,
}

impl CohortRow {
    pub fn into_cohort(self, start_block: Block) -> Cohort {
        Cohort {
            number: self.number as u32,
            network: Network::from_id(self.network_id as u32),
            announcement_date: self.announcement_date,
            announcement_url: self.announcement_url,
            delegation_date: self.delegation_date,
            start_block,
        }
    }
}

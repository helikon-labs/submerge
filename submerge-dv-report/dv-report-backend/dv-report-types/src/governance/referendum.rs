use crate::substrate::block::Block;
use crate::substrate::track::Track;
use serde::Serialize;
use sqlx::FromRow;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Debug, PartialEq, Serialize, EnumIter)]
pub enum ReferendumStatus {
    Ongoing,
    Confirmed,
    Rejected,
    Cancelled,
    TimedOut,
    Killed,
}

impl ReferendumStatus {
    pub fn id(&self) -> u32 {
        match self {
            ReferendumStatus::Ongoing => 1,
            ReferendumStatus::Confirmed => 2,
            ReferendumStatus::Rejected => 3,
            ReferendumStatus::Cancelled => 4,
            ReferendumStatus::TimedOut => 5,
            ReferendumStatus::Killed => 6,
        }
    }

    pub fn from_id(id: u32) -> Self {
        match id {
            1 => ReferendumStatus::Ongoing,
            2 => ReferendumStatus::Confirmed,
            3 => ReferendumStatus::Rejected,
            4 => ReferendumStatus::Cancelled,
            5 => ReferendumStatus::TimedOut,
            6 => ReferendumStatus::Killed,
            _ => panic!("Unknown referendum status id: {id}"),
        }
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn name(&self) -> String {
        match self {
            ReferendumStatus::Ongoing => "Ongoing",
            ReferendumStatus::Confirmed => "Confirmed",
            ReferendumStatus::Rejected => "Rejected",
            ReferendumStatus::Cancelled => "Cancelled",
            ReferendumStatus::TimedOut => "Timed Out",
            ReferendumStatus::Killed => "Killed",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferendumStatusRow {
    pub id: i32,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct Referendum {
    pub network_id: u32,
    pub index: u32,
    pub track: Track,
    pub submission_block: Block,
    pub status: ReferendumStatus,
}

#[derive(Clone, Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferendumRow {
    pub network_id: i32,
    pub index: i32,
    pub track_id: i32,
    pub submission_block_hash: String,
    pub status_id: i32,
    pub is_retracted: bool,
}

use std::fmt::{Display, Formatter};

use frame_system::Phase;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::Signature;

use crate::process::decode::Call;

pub mod legacy;
pub mod metadata;

#[derive(Clone, Debug, Serialize)]
pub struct Extrinsic {
    pub index: u32,
    pub trace_index: Option<u32>,
    pub hash: [u8; 32],
    pub signature: Option<Signature>,
    pub version: u8,
    pub is_successful: bool,
    pub call: Call,
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub trace_index: Option<u32>,
    pub pallet_index: u8,
    pub pallet_name: String,
    pub pallet_event_index: u8,
    pub pallet_event_name: String,
    pub index: u32,
    pub phase: Phase,
    pub args: JSONValue,
}

#[derive(Clone, Copy, Debug, sqlx::Type, Serialize, Deserialize, Eq, PartialEq)]
#[sqlx(type_name = "BLOCK_STATUS", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BlockStatus {
    Proposed,
    Pruned,
    Finalized,
}

impl Display for BlockStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BlockStatus::Proposed => "proposed",
                BlockStatus::Pruned => "pruned",
                BlockStatus::Finalized => "finalized",
            }
        )
    }
}

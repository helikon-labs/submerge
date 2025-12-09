use std::fmt::{Display, Formatter};

use decode::Value;
use frame_system::Phase;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::signature::Signature;
use utoipa::ToSchema;

pub(crate) mod api;
pub mod decode;
pub mod legacy;
pub mod metadata;
pub mod persistence;

pub struct GenesisItem {
    pub key_prefix: Vec<u8>,
    pub key_params: Option<Vec<u8>>,
    pub value: Vec<u8>,
    pub metadata_storage_item_id: Option<u32>,
    pub is_known_key: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Call {
    pub pallet_index: u8,
    pub pallet_name: String,
    pub pallet_call_index: u8,
    pub pallet_call_name: String,
    pub args: Value,
}

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

/// Block status.
/// - `finalized`: Blocks that have been finalized on chain.
/// - `proposed`: Blocks that have been authored but not yet finalized.
/// - `pruned`: Blocks that have been removed or are no longer accessible.
#[derive(Clone, Copy, Debug, sqlx::Type, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
#[sqlx(type_name = "BLOCK_STATUS", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[schema(example = "finalized")]
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

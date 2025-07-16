use frame_system::Phase;
use serde::Serialize;
use serde_json::Value as JsonValue;
use submerge_base::types::substrate::Signature;

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
    pub calls: Vec<Call>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Call {
    pub pallet_index: u8,
    pub pallet_name: String,
    pub pallet_call_index: u8,
    pub pallet_call_name: String,
    pub extrinsic_index: u32,
    pub args: Option<JsonValue>,
    pub sub_calls: Vec<Call>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub trace_index: u32,
    pub pallet_index: u8,
    pub pallet_name: String,
    pub pallet_event_index: u8,
    pub pallet_event_name: String,
    pub index: u32,
    pub phase: Phase,
    pub args: JsonValue,
}

use frame_system::Phase;
use serde::Serialize;
use serde_json::Value as JsonValue;

pub mod legacy;
pub mod metadata;

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
pub struct Call {
    pub pallet_index: u8,
    pub pallet_name: String,
    pub pallet_call_index: u8,
    pub pallet_call_name: String,
    pub extrinsic_index: u32,
    pub args: JsonValue,
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

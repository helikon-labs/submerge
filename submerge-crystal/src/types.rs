use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use sp_runtime::generic::Phase;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Call {
    pub pallet_index: u8,
    pub pallet_name: String,
    pub call_index: u8,
    pub call_name: String,
    pub extrinsic_index: u32,
    pub args: JsonMap<String, JsonValue>,
    pub sub_calls: Vec<Call>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Event {
    pub pallet_index: u8,
    pub pallet_name: String,
    pub event_index: u8,
    pub event_name: String,
    pub extrinsic_index: Option<u32>,
    pub phase: Phase,
    pub args: JsonMap<String, JsonValue>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SystemHealth {
    pub peers: u32,
    pub is_syncing: bool,
    pub should_have_peers: bool,
}

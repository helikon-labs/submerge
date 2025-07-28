use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::VariantNames;
use strum_macros::VariantNames;
use submerge_util::substrate::storage::get_storage_plain_key;

#[derive(Debug)]
pub struct ParseStorageMethodError(String);

impl Display for ParseStorageMethodError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseStorageMethodError {}

impl FromStr for StorageMethod {
    type Err = ParseStorageMethodError;

    /// Get chain from string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Put" => Ok(Self::Put),
            "ChildPut" => Ok(Self::ChildPut),
            "ChildKill" => Ok(Self::ChildKill),
            "ClearPrefix" => Ok(Self::ClearPrefix),
            "ChildClearPrefix" => Ok(Self::ChildClearPrefix),
            "Append" => Ok(Self::Append),
            "Genesis" => Ok(Self::Genesis),
            _ => Err(ParseStorageMethodError(format!(
                "Unknown storage method: {s}"
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, VariantNames)]
pub enum StorageMethod {
    Put,
    ChildPut,
    ChildKill,
    ClearPrefix,
    ChildClearPrefix,
    Append,
    Genesis,
}

impl Display for StorageMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Put => "Put",
            Self::ChildPut => "ChildPut",
            Self::ChildKill => "ChildKill",
            Self::ClearPrefix => "ClearPrefix",
            Self::ChildClearPrefix => "ChildClearPrefix",
            Self::Append => "Append",
            Self::Genesis => "Genesis",
        };
        write!(f, "{str}")
    }
}

impl StorageMethod {
    pub fn names() -> Vec<String> {
        StorageMethod::VARIANTS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockTraceData {
    pub key: String,
    pub value: String,
    pub ext_id: String,
    pub method: StorageMethod,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTraceDataWrapper {
    #[serde(rename(deserialize = "stringValues"))]
    pub data: BlockTraceData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTraceEvent {
    pub target: String,
    #[serde(rename(deserialize = "data"))]
    pub data_wrapper: BlockTraceDataWrapper,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTrace {
    pub block_hash: String,
    pub parent_hash: String,
    pub tracing_targets: String,
    pub storage_keys: String,
    pub methods: String,
    pub events: Vec<BlockTraceEvent>,
}

impl BlockTrace {
    pub fn get_event_count(&self) -> anyhow::Result<u32> {
        let event_count_key = get_storage_plain_key("System", "EventCount");
        let mut event_count: u32 = 0;
        for trace in self.events.iter() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key == event_count_key && trace_data.value.to_lowercase() != "none" {
                let value = trace_data
                    .value
                    .trim_start_matches("Some(")
                    .trim_end_matches(")");
                let mut bytes: &[u8] = &hex::decode(value)?;
                event_count = Decode::decode(&mut bytes)?;
            }
        }
        Ok(event_count)
    }

    pub fn get_extrinsic_count(&self) -> anyhow::Result<u32> {
        let extrinsic_count_key = get_storage_plain_key("System", "ExtrinsicCount");
        let mut extrinsic_count: u32 = 0;
        for trace in self.events.iter() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key == extrinsic_count_key && trace_data.value.to_lowercase() != "none" {
                let value = trace_data
                    .value
                    .trim_start_matches("Some(")
                    .trim_end_matches(")");
                let mut bytes: &[u8] = &hex::decode(value)?;
                extrinsic_count = Decode::decode(&mut bytes)?;
            }
        }
        Ok(extrinsic_count)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTraceWrapper {
    pub block_trace: BlockTrace,
}

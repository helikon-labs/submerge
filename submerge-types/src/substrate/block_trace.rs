use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use strum::VariantNames;
use strum_macros::VariantNames;

#[derive(Serialize, Deserialize, Debug, VariantNames)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockTraceData {
    pub key: String,
    pub value: String,
    pub value_encoded: Option<String>,
    pub ext_id: String,
    pub method: StorageMethod,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTraceDataWrapper {
    #[serde(rename(deserialize = "stringValues"))]
    pub data: BlockTraceData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTraceEvent {
    pub target: String,
    #[serde(rename(deserialize = "data"))]
    pub data_wrapper: BlockTraceDataWrapper,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTrace {
    pub block_hash: String,
    pub parent_hash: String,
    pub tracing_targets: String,
    pub storage_keys: String,
    pub methods: String,
    pub events: Vec<BlockTraceEvent>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockTraceWrapper {
    pub block_trace: BlockTrace,
}

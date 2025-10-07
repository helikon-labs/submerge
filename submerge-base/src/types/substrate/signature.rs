use serde::Serialize;
use serde_json::Value as JSONValue;

#[derive(Clone, Debug, Serialize)]
pub struct Signature {
    pub signer: super::multi_address::MultiAddress,
    pub signature: sp_runtime::MultiSignature,
    pub extra: Option<JSONValue>,
}

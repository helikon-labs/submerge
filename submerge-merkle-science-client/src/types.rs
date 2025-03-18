use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use submerge_util::serde::{
    deserialize_empty_object_as_none, float_to_string, iso_8601_to_naive_datetime,
};

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum RiskLevel {
    NoRisk = 0,
    Info = 1,
    Caution = 2,
    Medium = 3,
    High = 4,
    Critical = 5,
}

#[derive(Deserialize, Debug)]
pub struct Risk {
    #[serde(rename(deserialize = "risk_level"))]
    pub level: RiskLevel,
    #[serde(rename(deserialize = "risk_level_verbose"))]
    pub description: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Blockchain {
    #[serde(rename(deserialize = "blockchain"))]
    pub id: u16,
    #[serde(rename(deserialize = "blockchain_verbose"))]
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct BlockchainListResponse {
    pub results: Vec<Blockchain>,
}

#[derive(Deserialize, Debug)]
pub struct Workspace {
    pub name: String,
    pub slug: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Tag {
    #[serde(rename(deserialize = "tag_type_verbose"))]
    pub tag_type: String,
    #[serde(rename(deserialize = "tag_subtype_verbose"))]
    pub tag_subtype: String,
    #[serde(rename(deserialize = "tag_name_verbose"))]
    pub tag_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Tags {
    #[serde(deserialize_with = "deserialize_empty_object_as_none")]
    pub owner: Option<Tag>,
    #[serde(deserialize_with = "deserialize_empty_object_as_none")]
    pub user: Option<Tag>,
}

#[derive(Deserialize, Debug)]
pub struct DigitalAsset {
    pub name: String,
    pub symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct Entity {
    #[serde(flatten)]
    pub tag: Tag,
    #[serde(deserialize_with = "float_to_string")]
    pub total_value_usd: String,
    pub exposure_type: String,
}

#[derive(Serialize, Debug)]
pub struct AddressScreeningRequest {
    pub identifier: String,
    pub blockchain: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    #[serde(rename(deserialize = "type"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_alerts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_tags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct AddressScreening {
    pub identifier: String,
    #[serde(flatten)]
    pub blockchain: Blockchain,
    pub customer_id: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub address_type: Option<u16>,
    #[serde(rename(deserialize = "type_verbose"))]
    pub address_type_verbose: Option<String>,
    pub total_incoming_value: String,
    pub total_incoming_value_usd: String,
    pub total_outgoing_value: String,
    pub total_outgoing_value_usd: String,
    pub balance: f64,
    #[serde(deserialize_with = "iso_8601_to_naive_datetime")]
    pub earliest_transaction_time: NaiveDateTime,
    #[serde(deserialize_with = "iso_8601_to_naive_datetime")]
    pub latest_transaction_time: NaiveDateTime,
    #[serde(flatten)]
    pub risk: Risk,
    #[serde(deserialize_with = "iso_8601_to_naive_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(deserialize_with = "iso_8601_to_naive_datetime")]
    pub updated_at: NaiveDateTime,
    pub workspace: Option<Workspace>,
    #[serde(rename(deserialize = "originator"))]
    pub originators: Vec<Entity>,
    #[serde(rename(deserialize = "beneficiary"))]
    pub beneficiaries: Vec<Entity>,
    pub tags: Tags,
    pub digital_assets: Vec<DigitalAsset>,
    pub custom_tags: Vec<String>,
    pub is_megahub: bool,
}

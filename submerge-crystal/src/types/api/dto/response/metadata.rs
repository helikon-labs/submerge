use serde::Serialize;
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::api::dto::{
    pagination::PaginationData,
    response::{
        example::metadata::{metadata_pallet_example, metadata_summary_example},
        hex::HexString,
        schema::metadata::metadata_constant_value_schema,
    },
};

/// Basic fields to represent a runtime version metadata.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = MetadataSummary,
    example = metadata_summary_example,
)]
pub(crate) struct MetadataSummaryDTO {
    /// Metadata runtime spec version.
    #[schema(example = 1001)]
    pub spec_version: u32,
    /// Metadata version.
    #[schema(example = 14)]
    pub metadata_version: u32,
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
    description = "Response with a paginated list of metadata summaries.",
)]
pub(crate) struct PaginatedMetadataList {
    #[schema(example = json!([metadata_summary_example()]))]
    pub data: Vec<MetadataSummaryDTO>,
    pub pagination: PaginationData,
}

/// Parsed metadata specification complete with its pallets.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = Metadata)]
pub(crate) struct MetadataDTO {
    /// Metadata runtime spec version.
    #[schema(example = 1001)]
    pub spec_version: u32,
    /// Metadata version.
    #[schema(example = 14)]
    pub metadata_version: u32,
    /// Metadata pallets.
    #[schema(example = json!([metadata_pallet_example()]))]
    pub pallets: Vec<MetadataPalletDTO>,
}

/// Original Substrate runtime metadata in JSON representation.
#[derive(Debug, Serialize, ToSchema)]
#[schema(value_type = Value)]
pub(crate) struct MetadataJSON(pub JSONValue);

/// The summary of a pallet defined in metadata, with only its index and name.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = MetadataPalletSummary)]
pub(crate) struct MetadataPalletSummaryDTO {
    /// Pallet's index inside the metadata.
    #[schema(example = 53)]
    pub index: u32,
    /// Pallet's name.
    #[schema(example = "Balances")]
    pub name: String,
}

#[derive(Debug, Serialize, ToResponse)]
#[response(
    description = "List of summaries of matching metadata pallets.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct MetadataPalletSummaryList(pub Vec<MetadataPalletSummaryDTO>);

/// A pallet defined in metadata, with its calls, constants, errors, events, and storage items.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = MetadataPallet,
    example = metadata_pallet_example,
)]
pub(crate) struct MetadataPalletDTO {
    /// Pallet's index inside the metadata.
    #[schema(example = 53)]
    pub index: u32,
    /// Pallet's name.
    #[schema(example = "Balances")]
    pub name: String,
    /// Pallet's calls.
    pub calls: Vec<MetadataCallDTO>,
    /// Pallet's constants.
    pub constants: Vec<MetadataConstantDTO>,
    /// Pallet's errors.
    pub errors: Vec<MetadataErrorDTO>,
    /// Pallet's events.
    pub events: Vec<MetadataEventDTO>,
    /// Pallet's storage items.
    pub storage_items: Vec<MetadataStorageItemDTO>,
}

/// Metadata item documentation. One item per line in the array.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[schema(
    example = json!(["Call documentation line 1.", "Call documentation line 2."]),
)]
pub(crate) struct MetadataItemDocumentation(pub Vec<String>);

#[derive(Clone, Debug, Serialize, ToResponse)]
#[response(
    description = "All calls in a pallet.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct MetadataPalletCallList(pub Vec<MetadataCallDTO>);

/// A call defined in a metadata pallet.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = MetadataCall)]
pub(crate) struct MetadataCallDTO {
    /// Call's index inside the metadata pallet.
    #[schema(example = 37)]
    pub index: u32,
    /// Call name. Camel case.
    #[schema(example = "Transfer")]
    pub name: String,
    /// Call documentation. One item per line in the array.
    pub docs: MetadataItemDocumentation,
}

#[derive(Clone, Debug, Serialize, ToResponse)]
#[response(
    description = "All constants in a pallet.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct MetadataPalletConstantList(pub Vec<MetadataConstantDTO>);

/// A constant defined in a metadata pallet.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = MetadataConstant)]
pub(crate) struct MetadataConstantDTO {
    /// Constant's index inside the metadata pallet.
    #[schema(example = 7)]
    pub index: u32,
    /// Constant name. Camel case.
    #[schema(example = "BountyDepositBase")]
    pub name: String,
    /// Id of the constant's type.
    #[schema(required = false, nullable = false, example = 3)]
    pub type_id: Option<u32>,
    /// Name of the constant's type.
    #[schema(example = "TypeName")]
    pub type_name: String,
    /// Value of the constant. SCALE-encoded hexadecimal string.
    #[schema(example = "0x00e40b54020000000000000000000000")]
    pub value_hex: HexString,
    /// Value of the constant's type in JSON format.
    /// Only available in metadata version 14 and higher.
    #[schema(
        required = false,
        nullable = false,
        schema_with = metadata_constant_value_schema,
    )]
    pub value: Option<JSONValue>,
    /// Constant documentation. One item per line in the array.
    pub docs: MetadataItemDocumentation,
}

#[derive(Clone, Debug, Serialize, ToResponse)]
#[response(
    description = "All errors in a pallet.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct MetadataPalletErrorList(pub Vec<MetadataErrorDTO>);

/// An error defined in a metadata pallet.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = MetadataError)]
pub(crate) struct MetadataErrorDTO {
    /// Error's index inside the metadata pallet.
    #[schema(example = 17)]
    pub index: u32,
    /// Error name. Camel case.
    #[schema(example = "AssignmentsEmpty")]
    pub name: String,
    /// Error documentation. One item per line in the array.
    pub docs: MetadataItemDocumentation,
}

#[derive(Clone, Debug, Serialize, ToResponse)]
#[response(
    description = "All events in a pallet.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct MetadataPalletEventList(pub Vec<MetadataEventDTO>);

/// An event defined in a metadata pallet.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = MetadataEvent)]
pub(crate) struct MetadataEventDTO {
    /// Event's index inside the metadata pallet.
    #[schema(example = 21)]
    pub index: u32,
    /// Event name. Camel case.
    #[schema(example = "CandidateBacked")]
    pub name: String,
    /// Event documentation. One item per line in the array.
    pub docs: MetadataItemDocumentation,
}

#[derive(Clone, Debug, Serialize, ToResponse)]
#[response(
    description = "All storage items in a pallet.",
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub(crate) struct MetadataPalletStorageItemList(pub Vec<MetadataStorageItemDTO>);

/// A storage item defined in a metadata pallet.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = MetadataStorageItem)]
pub(crate) struct MetadataStorageItemDTO {
    /// Storage item's index inside the metadata pallet.
    #[schema(example = 0)]
    pub index: u32,
    /// Storage item name. Camel case.
    #[schema(example = "Account")]
    pub name: String,
    /// Storage item Substrate storage key prefix.
    pub key_prefix: HexString,
    /// Storage item documentation. One item per line in the array.
    pub docs: MetadataItemDocumentation,
}

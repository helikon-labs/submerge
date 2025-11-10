use frame_metadata::v14::{PalletMetadata as PalletMetadataV14, RuntimeMetadataV14};
use scale_info::{form::PortableForm, PortableType, Variant};

const UNCHECKED_EXTRINSIC_TYPE_PATH: &str =
    "sp_runtime::generic::unchecked_extrinsic::UncheckedExtrinsic";

pub fn get_metadata_type_by_id(
    metadata_v14: &RuntimeMetadataV14,
    type_id: u32,
) -> Option<&PortableType> {
    metadata_v14.types.types.iter().find(|ty| ty.id == type_id)
}

pub fn get_extrinsic_type(metadata_v14: &RuntimeMetadataV14) -> anyhow::Result<&PortableType> {
    let extrinsic_type = metadata_v14.types.types
        .iter()
        .find(|ty| ty.ty.path.segments.join("::").eq_ignore_ascii_case(UNCHECKED_EXTRINSIC_TYPE_PATH))
        .ok_or(anyhow::Error::msg(format!("Extrinsic type with path {UNCHECKED_EXTRINSIC_TYPE_PATH} not found in metadata type registry.")))?;
    get_metadata_type_by_id(metadata_v14, extrinsic_type.id).ok_or(anyhow::Error::msg(format!(
        "Extrinsic type with id {} not found in metadata.",
        extrinsic_type.id
    )))
}

pub fn get_extrinsic_signer_address_type(
    metadata_v14: &RuntimeMetadataV14,
) -> anyhow::Result<&PortableType> {
    let address_type_id = get_extrinsic_type(metadata_v14)?
        .ty
        .type_params
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("address"))
        .ok_or(anyhow::Error::msg(
            "Address type not found in extrinsic type params.",
        ))?
        .ty
        .ok_or(anyhow::Error::msg(
            "Address type is null in extrinsic type params.",
        ))?
        .id;
    get_metadata_type_by_id(metadata_v14, address_type_id)
        .ok_or(anyhow::Error::msg("Address type not found in metadata."))
}

pub fn get_extrinsic_signature_type(
    metadata_v14: &RuntimeMetadataV14,
) -> anyhow::Result<&PortableType> {
    let address_type_id = get_extrinsic_type(metadata_v14)?
        .ty
        .type_params
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("signature"))
        .ok_or(anyhow::Error::msg(
            "Signature type not found in extrinsic type params.",
        ))?
        .ty
        .ok_or(anyhow::Error::msg(
            "Signature type is null in extrinsic type params.",
        ))?
        .id;
    get_metadata_type_by_id(metadata_v14, address_type_id)
        .ok_or(anyhow::Error::msg("Signature type not found in metadata."))
}

pub fn get_pallet_metadata(
    metadata_v14: &RuntimeMetadataV14,
    pallet_index: u8,
) -> Option<&PalletMetadataV14<PortableForm>> {
    metadata_v14
        .pallets
        .iter()
        .find(|metadata_pallet| metadata_pallet.index == pallet_index)
}

fn get_pallet_events_type<'a>(
    metadata_v14: &'a RuntimeMetadataV14,
    pallet_metadata: &PalletMetadataV14<PortableForm>,
) -> Option<&'a PortableType> {
    if let Some(pallet_event_type) = &pallet_metadata.event {
        let type_id = pallet_event_type.ty.id;
        get_metadata_type_by_id(metadata_v14, type_id)
    } else {
        None
    }
}

pub fn get_event_variant<'a>(
    metadata_v14: &'a RuntimeMetadataV14,
    pallet_metadata: &PalletMetadataV14<PortableForm>,
    event_index: u8,
) -> anyhow::Result<Option<&'a Variant<PortableForm>>> {
    if let Some(events_type) = get_pallet_events_type(metadata_v14, pallet_metadata) {
        let event_variant = match &events_type.ty.type_def {
            scale_info::TypeDef::Variant(variant) => variant
                .variants
                .iter()
                .find(|variant| variant.index == event_index),
            _ => {
                anyhow::bail!(format!(
                    "Unexpected non-variant events type: {:?}",
                    events_type.ty.type_def
                ));
            }
        };
        Ok(event_variant)
    } else {
        anyhow::bail!("Events type not found in pallet.")
    }
}

pub fn get_storage_item_type_by_name<'a>(
    metadata_v14: &'a RuntimeMetadataV14,
    pallet_name: &'a str,
    pallet_storage_item_name: &'a str,
) -> Option<&'a PortableType> {
    let pallet = metadata_v14
        .pallets
        .iter()
        .find(|pallet| pallet.name.eq_ignore_ascii_case(pallet_name))?;
    let Some(storage_item) = &pallet.storage else {
        return None;
    };
    let storage_item = storage_item
        .entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(pallet_storage_item_name))?;
    let type_id = match &storage_item.ty {
        frame_metadata::v14::StorageEntryType::Plain(a) => a.id,
        frame_metadata::v14::StorageEntryType::Map { value, .. } => value.id,
    };
    get_metadata_type_by_id(metadata_v14, type_id)
}

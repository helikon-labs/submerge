use frame_metadata::{
    decode_different::{DecodeDifferent, DecodeDifferentStr},
    v14::{PalletMetadata, RuntimeMetadataV14},
    RuntimeMetadata,
};
use scale_info::{form::PortableForm, PortableType, Variant};

pub fn get_metadata_version(metadata: &RuntimeMetadata) -> u32 {
    match &metadata {
        RuntimeMetadata::V8(_) => 8,
        RuntimeMetadata::V9(_) => 9,
        RuntimeMetadata::V10(_) => 10,
        RuntimeMetadata::V11(_) => 11,
        RuntimeMetadata::V12(_) => 12,
        RuntimeMetadata::V13(_) => 13,
        RuntimeMetadata::V14(_) => 14,
        RuntimeMetadata::V15(_) => 15,
        RuntimeMetadata::V16(_) => 16,
        _ => unimplemented!("Unsupported metadata version."),
    }
}

pub fn get_decode_different_string(value: &DecodeDifferentStr) -> String {
    match value {
        DecodeDifferent::Encode(name) => name.to_string(),
        DecodeDifferent::Decoded(name) => name.clone(),
    }
}

pub fn get_metadata_type_by_id(
    metadata_v14: &RuntimeMetadataV14,
    type_id: u32,
) -> Option<&PortableType> {
    metadata_v14.types.types.iter().find(|ty| ty.id == type_id)
}

fn get_extrinsic_type(metadata_v14: &RuntimeMetadataV14) -> anyhow::Result<&PortableType> {
    let extrinsic_type = get_metadata_type_by_id(metadata_v14, metadata_v14.extrinsic.ty.id)
        .ok_or(anyhow::Error::msg("Extrinsic type not found in metadata."))?;
    Ok(extrinsic_type)
}

pub fn get_extrinsic_extra_type(
    metadata: &RuntimeMetadata,
) -> anyhow::Result<Option<&PortableType>> {
    match metadata {
        RuntimeMetadata::V14(metadata_v14) => {
            let extrinsic_type = get_extrinsic_type(metadata_v14)?;
            if let Some(ty) = extrinsic_type
                .ty
                .type_params
                .iter()
                .find(|p| p.name.to_lowercase() == "extra")
            {
                if let Some(ty) = ty.ty {
                    Ok(get_metadata_type_by_id(metadata_v14, ty.id))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        _ => anyhow::bail!(format!(
            "Unsupported metadata version: {}",
            get_metadata_version(metadata)
        )),
    }
}

pub fn get_runtime_call_type(metadata: &RuntimeMetadata) -> anyhow::Result<&PortableType> {
    match metadata {
        RuntimeMetadata::V14(metadata_v14) => {
            let extrinsic_type = get_extrinsic_type(metadata_v14)?;
            let call_type_id = extrinsic_type
                .ty
                .type_params
                .iter()
                .find(|p| p.name.to_lowercase() == "call")
                .ok_or(anyhow::Error::msg("Call type not found in metadata."))?
                .ty
                .ok_or(anyhow::Error::msg("Call type not found in metadata."))?
                .id;
            get_metadata_type_by_id(metadata_v14, call_type_id)
                .ok_or(anyhow::Error::msg("Call type not found in metadata."))
        }
        _ => anyhow::bail!(format!(
            "Unsupported metadata version: {}",
            get_metadata_version(metadata)
        )),
    }
}

pub fn get_pallet_metadata(
    metadata: &RuntimeMetadataV14,
    pallet_index: u8,
) -> Option<&PalletMetadata<PortableForm>> {
    metadata
        .pallets
        .iter()
        .find(|metadata_pallet| metadata_pallet.index == pallet_index)
}

fn get_pallet_events_type<'a>(
    metadata: &'a RuntimeMetadataV14,
    pallet_metadata: &PalletMetadata<PortableForm>,
) -> Option<&'a PortableType> {
    if let Some(pallet_event_type) = &pallet_metadata.event {
        let type_id = pallet_event_type.ty.id;
        get_metadata_type_by_id(metadata, type_id)
    } else {
        None
    }
}

pub fn get_event_variant<'a>(
    metadata: &'a RuntimeMetadataV14,
    pallet_metadata: &PalletMetadata<PortableForm>,
    event_index: u8,
) -> anyhow::Result<Option<&'a Variant<PortableForm>>> {
    if let Some(events_type) = get_pallet_events_type(metadata, pallet_metadata) {
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

pub fn get_signed_extensions(metadata_v14: &RuntimeMetadataV14) -> Vec<String> {
    metadata_v14
        .extrinsic
        .signed_extensions
        .iter()
        .map(|e| e.identifier.to_string())
        .collect()
}

pub fn get_block_weight_type(
    metadata_v14: &RuntimeMetadataV14,
) -> anyhow::Result<Option<&PortableType>> {
    if let Some(pallet) = metadata_v14
        .pallets
        .iter()
        .find(|p| p.name.to_lowercase() == "system")
    {
        if let Some(storage) = &pallet.storage {
            if let Some(entry) = storage
                .entries
                .iter()
                .find(|s| s.name.to_lowercase() == "blockweight")
            {
                match &entry.ty {
                    frame_metadata::v16::StorageEntryType::Plain(ty) => {
                        return Ok(get_metadata_type_by_id(metadata_v14, ty.id))
                    }
                    frame_metadata::v16::StorageEntryType::Map { .. } => (),
                }
            }
        }
    }
    Ok(None)
}

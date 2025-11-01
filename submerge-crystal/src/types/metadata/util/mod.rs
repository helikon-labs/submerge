use frame_metadata::{
    decode_different::{DecodeDifferent, DecodeDifferentStr},
    RuntimeMetadata,
};
use scale_info::PortableType;

pub mod v14;
pub mod v15;

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

pub fn get_extrinsic_signer_address_type(
    metadata: &RuntimeMetadata,
) -> anyhow::Result<&PortableType> {
    match metadata {
        RuntimeMetadata::V14(metadata_v14) => v14::get_extrinsic_signer_address_type(metadata_v14),
        RuntimeMetadata::V15(metadata_v15) => v15::get_extrinsic_signer_address_type(metadata_v15),
        _ => anyhow::bail!(format!(
            "Unsupported metadata version: {}",
            get_metadata_version(metadata)
        )),
    }
}

pub fn get_extrinsic_extra_type(
    metadata: &RuntimeMetadata,
) -> anyhow::Result<Option<&PortableType>> {
    match metadata {
        RuntimeMetadata::V14(metadata_v14) => {
            let extrinsic_type = v14::get_extrinsic_type(metadata_v14)?;
            if let Some(ty) =
                extrinsic_type.ty.type_params.iter().find(|p| {
                    p.name.to_lowercase() == "extra" || p.name.to_lowercase() == "extension"
                })
            {
                if let Some(ty) = ty.ty {
                    Ok(v14::get_metadata_type_by_id(metadata_v14, ty.id))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        RuntimeMetadata::V15(metadata_v15) => Ok(v15::get_metadata_type_by_id(
            metadata_v15,
            metadata_v15.extrinsic.extra_ty.id,
        )),
        _ => anyhow::bail!(format!(
            "Unsupported metadata version: {}",
            get_metadata_version(metadata)
        )),
    }
}

pub fn get_runtime_call_type(metadata: &RuntimeMetadata) -> anyhow::Result<&PortableType> {
    match metadata {
        RuntimeMetadata::V14(metadata_v14) => {
            let extrinsic_type = v14::get_extrinsic_type(metadata_v14)?;
            let call_type_id = extrinsic_type
                .ty
                .type_params
                .iter()
                .find(|p| p.name.to_lowercase() == "call")
                .ok_or(anyhow::Error::msg("Call type not found in metadata."))?
                .ty
                .ok_or(anyhow::Error::msg("Call type not found in metadata."))?
                .id;
            v14::get_metadata_type_by_id(metadata_v14, call_type_id)
                .ok_or(anyhow::Error::msg("Call type not found in metadata."))
        }
        RuntimeMetadata::V15(metadata_v15) => {
            v15::get_metadata_type_by_id(metadata_v15, metadata_v15.extrinsic.call_ty.id)
                .ok_or(anyhow::Error::msg("Call type not found in metadata."))
        }
        _ => anyhow::bail!(format!(
            "Unsupported metadata version: {}",
            get_metadata_version(metadata)
        )),
    }
}

pub fn get_signed_extensions(metadata: &RuntimeMetadata) -> anyhow::Result<Vec<String>> {
    match metadata {
        RuntimeMetadata::V14(metadata_v14) => Ok(metadata_v14
            .extrinsic
            .signed_extensions
            .iter()
            .map(|e| e.identifier.to_string())
            .collect()),
        RuntimeMetadata::V15(metadata_v15) => Ok(metadata_v15
            .extrinsic
            .signed_extensions
            .iter()
            .map(|e| e.identifier.to_string())
            .collect()),
        _ => anyhow::bail!(format!(
            "Unsupported metadata version: {}",
            get_metadata_version(metadata)
        )),
    }
}

pub fn get_block_weight_type(metadata: &RuntimeMetadata) -> anyhow::Result<Option<&PortableType>> {
    match metadata {
        RuntimeMetadata::V14(metadata_v14) => {
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
                                return Ok(v14::get_metadata_type_by_id(metadata_v14, ty.id))
                            }
                            frame_metadata::v16::StorageEntryType::Map { .. } => (),
                        }
                    }
                }
            }
        }
        RuntimeMetadata::V15(metadata_v15) => {
            if let Some(pallet) = metadata_v15
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
                                return Ok(v15::get_metadata_type_by_id(metadata_v15, ty.id))
                            }
                            frame_metadata::v16::StorageEntryType::Map { .. } => (),
                        }
                    }
                }
            }
        }
        _ => anyhow::bail!(format!(
            "Unsupported metadata version: {}",
            get_metadata_version(metadata)
        )),
    }
    Ok(None)
}

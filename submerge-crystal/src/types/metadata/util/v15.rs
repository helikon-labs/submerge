use frame_metadata::v15::{PalletMetadata as PalletMetadataV15, RuntimeMetadataV15};
use scale_info::{form::PortableForm, PortableType, Variant};

pub fn get_metadata_type_by_id(
    metadata_v15: &RuntimeMetadataV15,
    type_id: u32,
) -> Option<&PortableType> {
    metadata_v15.types.types.iter().find(|ty| ty.id == type_id)
}

pub fn get_pallet_metadata(
    metadata_v15: &RuntimeMetadataV15,
    pallet_index: u8,
) -> Option<&PalletMetadataV15<PortableForm>> {
    metadata_v15
        .pallets
        .iter()
        .find(|metadata_pallet| metadata_pallet.index == pallet_index)
}

fn get_pallet_events_type<'a>(
    metadata_v15: &'a RuntimeMetadataV15,
    pallet_metadata: &PalletMetadataV15<PortableForm>,
) -> Option<&'a PortableType> {
    if let Some(pallet_event_type) = &pallet_metadata.event {
        let type_id = pallet_event_type.ty.id;
        get_metadata_type_by_id(metadata_v15, type_id)
    } else {
        None
    }
}

pub fn get_event_variant<'a>(
    metadata_v15: &'a RuntimeMetadataV15,
    pallet_metadata: &PalletMetadataV15<PortableForm>,
    event_index: u8,
) -> anyhow::Result<Option<&'a Variant<PortableForm>>> {
    if let Some(events_type) = get_pallet_events_type(metadata_v15, pallet_metadata) {
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

pub fn get_extrinsic_signer_address_type(
    metadata_v15: &RuntimeMetadataV15,
) -> anyhow::Result<&PortableType> {
    get_metadata_type_by_id(metadata_v15, metadata_v15.extrinsic.address_ty.id).ok_or(
        anyhow::Error::msg("Extrinsic signer address type not found in metadata."),
    )
}

pub fn get_extrinsic_signature_type(
    metadata_v15: &RuntimeMetadataV15,
) -> anyhow::Result<&PortableType> {
    get_metadata_type_by_id(metadata_v15, metadata_v15.extrinsic.signature_ty.id).ok_or(
        anyhow::Error::msg("Extrinsic signature type not found in metadata."),
    )
}

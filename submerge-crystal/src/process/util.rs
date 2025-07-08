use frame_metadata::v14::{PalletMetadata, RuntimeMetadataV14};
use parity_scale_codec::Decode;
use scale_info::{form::PortableForm, PortableType, Variant};
use submerge_base::types::substrate::block_trace::BlockTrace;
use submerge_util::substrate::storage::get_storage_plain_key;

pub fn get_extrinsic_count(trace: &BlockTrace) -> anyhow::Result<u32> {
    let extrinsic_count_key = get_storage_plain_key("System", "ExtrinsicCount");
    let mut extrinsic_count: u32 = 0;
    for trace in trace.events.iter() {
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

pub fn get_event_count(trace: &BlockTrace) -> anyhow::Result<u32> {
    let event_count_key = get_storage_plain_key("System", "EventCount");
    let mut event_count: u32 = 0;
    for trace in trace.events.iter() {
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

pub fn get_pallet_metadata(
    metadata: &RuntimeMetadataV14,
    pallet_index: u8,
) -> Option<&PalletMetadata<PortableForm>> {
    metadata
        .pallets
        .iter()
        .find(|metadata_pallet| metadata_pallet.index == pallet_index)
}

fn get_event_type<'a>(
    metadata: &'a RuntimeMetadataV14,
    pallet_metadata: &PalletMetadata<PortableForm>,
) -> Option<&'a PortableType> {
    if let Some(pallet_event_type) = &pallet_metadata.event {
        let type_id = pallet_event_type.ty.id;
        metadata.types.types.iter().find(|ty| ty.id == type_id)
    } else {
        None
    }
}

pub fn get_event_variant<'a>(
    metadata: &'a RuntimeMetadataV14,
    pallet_metadata: &PalletMetadata<PortableForm>,
    event_index: u8,
) -> anyhow::Result<Option<&'a Variant<PortableForm>>> {
    if let Some(event_type) = get_event_type(metadata, pallet_metadata) {
        let event_variant = match &event_type.ty.type_def {
            scale_info::TypeDef::Variant(variant) => variant
                .variants
                .iter()
                .find(|variant| variant.index == event_index),
            _ => {
                anyhow::bail!(format!(
                    "Unexpected non-variant event type: {:?}",
                    event_type.ty.type_def
                ));
            }
        };
        Ok(event_variant)
    } else {
        anyhow::bail!("Event type not found in pallet.")
    }
}

use convert_case::{Case, Casing};
use frame_metadata::RuntimeMetadata;
use frame_system::Phase;
use parity_scale_codec::{Compact, Decode};
use serde_json::Value as JsonValue;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::{
    block::BlockHeader,
    block_trace::{BlockTrace, StorageMethod},
};
use submerge_util::substrate::storage::get_storage_plain_key;

use crate::{
    persistence::CrystalPostgreSQLStorage,
    process::{decode::JsonValueVisitor, metadata::get_metadata_version, BlockProcessor},
    types::Event,
    util::{get_event_variant, get_pallet_metadata},
};

impl BlockProcessor {
    pub async fn get_events(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
        trace: &BlockTrace,
    ) -> anyhow::Result<Vec<Event>> {
        let mut events = Vec::new();
        let metadata_version = get_metadata_version(metadata);
        let events_key = get_storage_plain_key("System", "Events");
        let mut processed_events_hex = String::new();
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key != events_key || trace_data.value.to_lowercase() == "none" {
                continue;
            }
            let value = trace_data
                .value
                .trim_start_matches("Some(")
                .trim_end_matches(")");
            let value = match trace_data.method {
                StorageMethod::Put => {
                    let mut bytes: &[u8] = &hex::decode(value)?;
                    // skip event count
                    let _ = <Compact<u32>>::decode(&mut bytes)?.0;
                    // skip processed events
                    hex::encode(bytes)
                        .trim_start_matches(&processed_events_hex)
                        .to_string()
                }
                _ => value.to_string(),
            };
            let mut bytes: &[u8] = &hex::decode(&value)?;
            if metadata_version < 14 {
                let event = self
                    .legacy_decode_api_client
                    .decode_event(block_hash, spec_version, bytes)
                    .await?;
                let legacy_phase = event.get_phase()?;
                let phase = match legacy_phase.ty.to_lowercase().as_str() {
                    "initialization" => Phase::Initialization,
                    "finalization" => Phase::Finalization,
                    "applyextrinsic" => {
                        if let JsonValue::String(extrinsic_index) = legacy_phase.value {
                            let extrinsic_index: u32 = extrinsic_index.parse()?;
                            Phase::ApplyExtrinsic(extrinsic_index)
                        } else {
                            anyhow::bail!(
                                "Unexpected value for ApplyExtrinsic phase: {:?}",
                                legacy_phase.value
                            );
                        }
                    }
                    _ => anyhow::bail!(
                        "Unexpected phase :: {} {:?}",
                        legacy_phase.ty,
                        legacy_phase.value
                    ),
                };
                let pallet_name = event.event.pallet.to_case(Case::UpperCamel);
                let pallet_index = self
                    .postgres
                    .get_pallet_index_by_name(&pallet_name)
                    .await?
                    .ok_or(anyhow::Error::msg(format!(
                        "Pallet index not found in the database for pallet {pallet_name}."
                    )))?;
                let pallet_event_name = event.event.name.to_case(Case::UpperCamel);
                let pallet_event_index = self
                    .postgres
                    .get_pallet_event_index_by_name(pallet_index, &pallet_event_name)
                    .await?
                    .ok_or(anyhow::Error::msg(format!("Pallet event index not found in the database for event {pallet_name}.{pallet_event_name}.")))?;
                events.push(Event {
                    trace_index: trace_index as u32,
                    pallet_index,
                    pallet_name,
                    pallet_event_index,
                    pallet_event_name,
                    index: events.len() as u32,
                    phase,
                    args: event.event.data,
                });
                continue;
            }
            let phase = frame_system::Phase::decode(&mut bytes)?;
            let pallet_index: u8 = Decode::decode(&mut bytes)?;
            let pallet_event_index: u8 = Decode::decode(&mut bytes)?;
            match &metadata {
                RuntimeMetadata::V14(metadata) => {
                    let pallet_metadata = get_pallet_metadata(metadata, pallet_index)
                        .expect("Pallet not found in metadata.");
                    let event_variant =
                        get_event_variant(metadata, pallet_metadata, pallet_event_index)?
                            .expect("Event not found in pallet.");
                    let mut map = serde_json::Map::new();
                    for event_field in event_variant.fields.iter() {
                        let field_type = metadata
                            .types
                            .types
                            .iter()
                            .find(|metadata_type| metadata_type.id == event_field.ty.id)
                            .expect("Calls type not found in pallet.");
                        let visitor = JsonValueVisitor::new();
                        let value: JsonValue = scale_decode::visitor::decode_with_visitor(
                            &mut bytes,
                            field_type.id,
                            &metadata.types,
                            visitor,
                        )?;
                        if let Some(field_name) = &event_field.name {
                            map.insert(field_name.clone(), value);
                        } else if let Some(type_name) = &event_field.type_name {
                            map.insert(type_name.clone(), value);
                        } else {
                            map.insert("unnamed".to_string(), value);
                        }
                    }
                    let args = JsonValue::Object(map);
                    events.push(Event {
                        trace_index: trace_index as u32,
                        pallet_index,
                        pallet_name: pallet_metadata.name.to_case(Case::UpperCamel),
                        pallet_event_index,
                        pallet_event_name: event_variant.name.to_case(Case::UpperCamel),
                        index: events.len() as u32,
                        phase,
                        args,
                    });
                }
                _ => anyhow::bail!("Unsupported runtime metadata version."),
            }
            if let StorageMethod::Put = trace_data.method {
                processed_events_hex.push_str(value.as_str());
            }
        }
        Ok(events)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn process_events(
        &self,
        block_hash: &[u8],
        block_header: &BlockHeader,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        events: &[Event],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        for event in events.iter() {
            self.postgres
                .ingest_event(
                    block_hash,
                    block_header.get_number()?,
                    block_timestamp,
                    spec_version,
                    is_finalized,
                    event,
                    tx,
                )
                .await?;
        }
        Ok(())
    }
}

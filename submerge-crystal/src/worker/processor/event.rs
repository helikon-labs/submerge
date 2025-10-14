use convert_case::{Case, Casing};
use frame_metadata::RuntimeMetadata;
use frame_system::Phase;
use parity_scale_codec::{Compact, Decode};
use serde_json::Value as JSONValue;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::{
    block::BlockHeader,
    block_trace::{BlockTrace, StorageMethod},
};
use submerge_util::substrate::storage::get_storage_plain_key;

use crate::{
    persistence::CrystalPostgreSQLStorage,
    types::{
        decode::{Value, ValueVisitor},
        metadata::util::{get_metadata_version, get_runtime_call_type, v14, v15},
        persistence::EventRow,
        BlockStatus, Event, Extrinsic,
    },
    worker::processor::BlockProcessor,
};

impl BlockProcessor {
    async fn get_legacy_events_from_bytes(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Vec<Event>> {
        let legacy_decode_api_client = if let Some(client) = &self.legacy_decode_api_client {
            client
        } else {
            anyhow::bail!(
                "Legacy decode API client is not set. legacy_decode_api_url parameter not set."
            );
        };
        let legacy_events = legacy_decode_api_client
            .decode_events(block_hash, spec_version, bytes)
            .await?;
        let mut events = Vec::new();
        for event in legacy_events.iter() {
            let legacy_phase = event.get_phase()?;
            let phase = match legacy_phase.ty.to_lowercase().as_str() {
                "initialization" => Phase::Initialization,
                "finalization" => Phase::Finalization,
                "applyextrinsic" => {
                    if let JSONValue::String(extrinsic_index) = legacy_phase.value {
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
            let metadata = self.get_parsed_metadata(block_hash, spec_version).await?;
            let pallet_name = &event.event.pallet;
            let pallet = metadata
                .pallets
                .iter()
                .find(|pallet| pallet.name.to_lowercase() == pallet_name.to_lowercase())
                .ok_or(anyhow::Error::msg(format!(
                    "Pallet {pallet_name} not found in metadata database."
                )))?;
            let pallet_event_name = &event.event.name;
            let pallet_event = pallet
                .events
                .iter()
                .find(|event| event.name.to_lowercase() == pallet_event_name.to_lowercase())
                .ok_or(anyhow::Error::msg(format!(
                    "Event {pallet_event_name} not found in pallet {pallet_name} in metadata database."
                )))?;
            events.push(Event {
                trace_index: None,
                pallet_index: pallet.index,
                pallet_name: pallet_name.clone(),
                pallet_event_index: pallet_event.index,
                pallet_event_name: pallet_event_name.clone(),
                index: events.len() as u32,
                phase,
                args: event.event.data.clone(),
            });
        }
        Ok(events)
    }

    async fn get_events_from_bytes(
        &self,
        metadata: &RuntimeMetadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Vec<Event>> {
        let call_type = get_runtime_call_type(metadata)?;
        let event_count = <Compact<u32>>::decode(bytes)?.0;
        let mut events = Vec::new();
        for _ in 0..event_count {
            let phase = frame_system::Phase::decode(bytes)?;
            let pallet_index: u8 = Decode::decode(bytes)?;
            let pallet_event_index: u8 = Decode::decode(bytes)?;
            match &metadata {
                RuntimeMetadata::V14(metadata_v14) => {
                    let pallet_metadata = v14::get_pallet_metadata(metadata_v14, pallet_index)
                        .ok_or(anyhow::Error::msg("Pallet not found in metadata."))?;
                    let pallet_name = pallet_metadata.name.to_case(Case::UpperCamel);
                    let event_variant =
                        v14::get_event_variant(metadata_v14, pallet_metadata, pallet_event_index)?
                            .ok_or(anyhow::Error::msg("Event not found in pallet."))?;
                    let pallet_event_name = event_variant.name.to_case(Case::UpperCamel);

                    let mut map = serde_json::Map::new();
                    for event_field in event_variant.fields.iter() {
                        let visitor = ValueVisitor::new(call_type.id, None);
                        let value: Value = scale_decode::visitor::decode_with_visitor(
                            bytes,
                            event_field.ty.id,
                            &metadata_v14.types,
                            visitor,
                        )?;
                        if let Some(field_name) = &event_field.name {
                            map.insert(field_name.to_case(Case::Camel), value.into());
                        } else if let Some(type_name) = &event_field.type_name {
                            map.insert(type_name.clone(), value.into());
                        } else {
                            map.insert("unnamed".to_string(), value.into());
                        }
                    }
                    let args = JSONValue::Object(map);
                    let event = Event {
                        trace_index: None,
                        pallet_index,
                        pallet_name,
                        pallet_event_index,
                        pallet_event_name,
                        index: events.len() as u32,
                        phase,
                        args,
                    };
                    events.push(event);
                    let _topics = Vec::<sp_core::H256>::decode(bytes)?;
                }
                _ => anyhow::bail!("Unsupported runtime metadata version."),
            }
        }
        Ok(events)
    }

    pub async fn get_events_from_event_bytes(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
        bytes: Vec<u8>,
    ) -> anyhow::Result<Vec<Event>> {
        let metadata_version = get_metadata_version(metadata);
        let mut bytes: &[u8] = &bytes;
        let events = if metadata_version < 14 {
            self.get_legacy_events_from_bytes(block_hash, spec_version, &mut bytes)
                .await?
        } else {
            self.get_events_from_bytes(metadata, &mut bytes).await?
        };
        Ok(events)
    }

    pub async fn get_events_from_trace(
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
                let legacy_decode_api_client = if let Some(client) = &self.legacy_decode_api_client
                {
                    client
                } else {
                    anyhow::bail!("Legacy decode API client is not set. legacy_decode_api_url parameter not set.");
                };
                let event = legacy_decode_api_client
                    .decode_event(block_hash, spec_version, bytes)
                    .await?;
                let legacy_phase = event.get_phase()?;
                let phase = match legacy_phase.ty.to_lowercase().as_str() {
                    "initialization" => Phase::Initialization,
                    "finalization" => Phase::Finalization,
                    "applyextrinsic" => {
                        if let JSONValue::String(extrinsic_index) = legacy_phase.value {
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

                let metadata = self.get_parsed_metadata(block_hash, spec_version).await?;
                let pallet_name = &event.event.pallet;
                let pallet = metadata
                    .pallets
                    .iter()
                    .find(|pallet| pallet.name.to_lowercase() == pallet_name.to_lowercase())
                    .ok_or(anyhow::Error::msg(format!(
                        "Pallet {pallet_name} not found in metadata database."
                    )))?;
                let pallet_event_name = &event.event.name;
                let pallet_event = pallet
                    .events
                    .iter()
                    .find(|event| event.name.to_lowercase() == pallet_event_name.to_lowercase())
                    .ok_or(anyhow::Error::msg(format!(
                        "Event {pallet_event_name} not found in pallet {pallet_name} in metadata database."
                    )))?;
                events.push(Event {
                    trace_index: Some(trace_index as u32),
                    pallet_index: pallet.index,
                    pallet_name: pallet.name.clone(),
                    pallet_event_index: pallet_event.index,
                    pallet_event_name: pallet_event.name.clone(),
                    index: events.len() as u32,
                    phase,
                    args: event.event.data,
                });
                continue;
            }
            let call_type = get_runtime_call_type(metadata)?;
            let phase = frame_system::Phase::decode(&mut bytes)?;
            let pallet_index: u8 = Decode::decode(&mut bytes)?;
            let pallet_event_index: u8 = Decode::decode(&mut bytes)?;
            match &metadata {
                RuntimeMetadata::V14(metadata_v14) => {
                    let pallet_metadata = v14::get_pallet_metadata(metadata_v14, pallet_index)
                        .ok_or(anyhow::Error::msg("Pallet not found in metadata."))?;
                    let event_variant =
                        v14::get_event_variant(metadata_v14, pallet_metadata, pallet_event_index)?
                            .ok_or(anyhow::Error::msg("Event not found in pallet."))?;
                    let mut map = serde_json::Map::new();

                    for event_field in event_variant.fields.iter() {
                        let visitor = ValueVisitor::new(call_type.id, None);
                        let value: Value = scale_decode::visitor::decode_with_visitor(
                            &mut bytes,
                            event_field.ty.id,
                            &metadata_v14.types,
                            visitor,
                        )?;
                        if let Some(field_name) = &event_field.name {
                            map.insert(field_name.to_case(Case::Camel), value.into());
                        } else if let Some(type_name) = &event_field.type_name {
                            map.insert(type_name.clone(), value.into());
                        } else {
                            map.insert("unnamed".to_string(), value.into());
                        }
                    }
                    let args = JSONValue::Object(map);
                    events.push(Event {
                        trace_index: Some(trace_index as u32),
                        pallet_index,
                        pallet_name: pallet_metadata.name.to_case(Case::UpperCamel),
                        pallet_event_index,
                        pallet_event_name: event_variant.name.to_case(Case::UpperCamel),
                        index: events.len() as u32,
                        phase,
                        args,
                    });
                }
                RuntimeMetadata::V15(metadata_v15) => {
                    let pallet_metadata = v15::get_pallet_metadata(metadata_v15, pallet_index)
                        .ok_or(anyhow::Error::msg("Pallet not found in metadata."))?;
                    let event_variant =
                        v15::get_event_variant(metadata_v15, pallet_metadata, pallet_event_index)?
                            .ok_or(anyhow::Error::msg("Event not found in pallet."))?;
                    let mut map = serde_json::Map::new();

                    for event_field in event_variant.fields.iter() {
                        let visitor = ValueVisitor::new(call_type.id, None);
                        let value: Value = scale_decode::visitor::decode_with_visitor(
                            &mut bytes,
                            event_field.ty.id,
                            &metadata_v15.types,
                            visitor,
                        )?;
                        if let Some(field_name) = &event_field.name {
                            map.insert(field_name.to_case(Case::Camel), value.into());
                        } else if let Some(type_name) = &event_field.type_name {
                            map.insert(type_name.clone(), value.into());
                        } else {
                            map.insert("unnamed".to_string(), value.into());
                        }
                    }
                    let args = JSONValue::Object(map);
                    events.push(Event {
                        trace_index: Some(trace_index as u32),
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

    pub async fn process_events(
        &self,
        block_hash: &[u8],
        block_header: &BlockHeader,
        block_timestamp: Option<u64>,
        spec_version: u32,
        block_status: BlockStatus,
        events: &[Event],
        extrinsics: &[Extrinsic],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let metadata = self.get_parsed_metadata(block_hash, spec_version).await?;
        let mut rows: Vec<EventRow> = Vec::new();
        for event in events.iter() {
            let pallet_event = metadata
                .get_pallet_by_index(event.pallet_index)
                .ok_or(anyhow::anyhow!(
                    "Pallet {} with index {} not found in database.",
                    event.pallet_name,
                    event.pallet_index
                ))?
                .get_event_by_index(event.pallet_event_index)
                .ok_or(anyhow::anyhow!(
                    "Event {} with index {} not found in database.",
                    event.pallet_event_name,
                    event.pallet_event_index
                ))?;
            rows.push(EventRow::from_block_event(
                block_hash,
                block_header.get_number()?,
                block_timestamp,
                spec_version,
                block_status,
                event,
                extrinsics,
                pallet_event.id,
            ));
        }
        self.postgres.ingest_events(&rows, tx).await?;
        Ok(())
    }
}

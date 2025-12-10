use std::sync::Arc;

use convert_case::{Case, Casing};
use frame_metadata::{v14::RuntimeMetadataV14, v15::RuntimeMetadataV15, RuntimeMetadata};
use frame_system::Phase;
use parity_scale_codec::{Compact, Decode};
use scale_info::{form::PortableForm, PortableRegistry, Variant};
use serde_json::Value as JSONValue;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::{
    block::BlockHeader,
    block_trace::{BlockTrace, StorageMethod},
};
use submerge_util::substrate::storage::get_storage_plain_key;

use crate::{
    api::legacy::LegacyDecodeAPIClient,
    persistence::CrystalPostgreSQLStorage,
    types::{
        decode::{Value, ValueVisitor},
        legacy::{LegacyEventPhase, LegacyEventWrapper},
        metadata::util::{get_metadata_version, get_runtime_call_type, v14, v15},
        persistence::EventRow,
        BlockStatus, Event, Extrinsic,
    },
    worker::{metadata_cache::get_parsed_metadata, processor::BlockProcessor},
};

const SYSTEM_PALLET_NAME: &str = "System";
const EVENTS_STORAGE_KEY: &str = "Events";
const SOME_PREFIX: &str = "Some(";
const SOME_SUFFIX: &str = ")";
const NONE_VALUE: &str = "none";
const METADATA_VERSION_LEGACY_THRESHOLD: u32 = 14;

fn decode_event(
    types: &PortableRegistry,
    call_type_id: u32,
    trace_index: Option<u32>,
    pallet_index: u8,
    pallet_name: &str,
    pallet_event_index: u8,
    pallet_event_name: &str,
    event_variant: &Variant<PortableForm>,
    phase: Phase,
    index: u32,
    bytes: &mut &[u8],
) -> anyhow::Result<Event> {
    let mut map = serde_json::Map::new();
    for event_field in event_variant.fields.iter() {
        let visitor = ValueVisitor::new(call_type_id, None);
        let value: Value =
            scale_decode::visitor::decode_with_visitor(bytes, event_field.ty.id, types, visitor)?;
        let field_key = event_field
            .name
            .as_ref()
            .map(|name| name.to_case(Case::Camel))
            .or_else(|| event_field.type_name.clone())
            .unwrap_or_else(|| format!("unnamed_{}", map.len()));
        map.insert(field_key, value.into());
    }
    let args = JSONValue::Object(map);
    Ok(Event {
        trace_index,
        pallet_index,
        pallet_name: pallet_name.to_string(),
        pallet_event_index,
        pallet_event_name: pallet_event_name.to_string(),
        index,
        phase,
        args,
    })
}

fn decode_metadata_v14_event(
    metadata_v14: &RuntimeMetadataV14,
    call_type_id: u32,
    trace_index: Option<u32>,
    pallet_index: u8,
    pallet_event_index: u8,
    phase: Phase,
    index: u32,
    bytes: &mut &[u8],
) -> anyhow::Result<Event> {
    let pallet_metadata = v14::get_pallet_metadata(metadata_v14, pallet_index)
        .ok_or(anyhow::Error::msg("Pallet not found in metadata."))?;
    let pallet_name = pallet_metadata.name.to_case(Case::UpperCamel);
    let event_variant = v14::get_event_variant(metadata_v14, pallet_metadata, pallet_event_index)?
        .ok_or(anyhow::Error::msg("Event not found in pallet."))?;
    let pallet_event_name = event_variant.name.to_case(Case::UpperCamel);

    decode_event(
        &metadata_v14.types,
        call_type_id,
        trace_index,
        pallet_index,
        &pallet_name,
        pallet_event_index,
        &pallet_event_name,
        event_variant,
        phase,
        index,
        bytes,
    )
}

fn decode_metadata_v15_event(
    metadata_v15: &RuntimeMetadataV15,
    call_type_id: u32,
    trace_index: Option<u32>,
    pallet_index: u8,
    pallet_event_index: u8,
    phase: Phase,
    index: u32,
    bytes: &mut &[u8],
) -> anyhow::Result<Event> {
    let pallet_metadata = v15::get_pallet_metadata(metadata_v15, pallet_index)
        .ok_or(anyhow::Error::msg("Pallet not found in metadata."))?;
    let pallet_name = pallet_metadata.name.to_case(Case::UpperCamel);
    let event_variant = v15::get_event_variant(metadata_v15, pallet_metadata, pallet_event_index)?
        .ok_or(anyhow::Error::msg("Event not found in pallet."))?;
    let pallet_event_name = event_variant.name.to_case(Case::UpperCamel);
    decode_event(
        &metadata_v15.types,
        call_type_id,
        trace_index,
        pallet_index,
        &pallet_name,
        pallet_event_index,
        &pallet_event_name,
        event_variant,
        phase,
        index,
        bytes,
    )
}

fn parse_legacy_phase(legacy_phase: &LegacyEventPhase) -> anyhow::Result<Phase> {
    match legacy_phase.ty.to_lowercase().as_str() {
        "initialization" => Ok(Phase::Initialization),
        "finalization" => Ok(Phase::Finalization),
        "applyextrinsic" => {
            let extrinsic_index = legacy_phase
                .value
                .as_str()
                .ok_or_else(|| {
                    anyhow::Error::msg(format!(
                        "Expected string for ApplyExtrinsic phase, got: {:?}",
                        legacy_phase.value
                    ))
                })?
                .parse()?;
            Ok(Phase::ApplyExtrinsic(extrinsic_index))
        }
        _ => anyhow::bail!(
            "Unexpected phase type: {} with value: {:?}",
            legacy_phase.ty,
            legacy_phase.value
        ),
    }
}

fn extract_trace_value(
    value: &str,
    method: &StorageMethod,
    processed_events_hex: &str,
) -> anyhow::Result<String> {
    let trimmed = value
        .trim_start_matches(SOME_PREFIX)
        .trim_end_matches(SOME_SUFFIX);
    match method {
        StorageMethod::Put => {
            let mut bytes: &[u8] = &hex::decode(trimmed)?;
            let _event_count = <Compact<u32>>::decode(&mut bytes)?.0;
            Ok(hex::encode(bytes)
                .trim_start_matches(processed_events_hex)
                .to_string())
        }
        _ => Ok(trimmed.to_string()),
    }
}

impl BlockProcessor {
    fn get_legacy_decode_client(&self) -> anyhow::Result<&Arc<LegacyDecodeAPIClient>> {
        self.legacy_decode_api_client.as_ref().ok_or_else(|| {
            anyhow::Error::msg(
                "Legacy decode API client is not set. legacy_decode_api_url parameter not set.",
            )
        })
    }

    async fn decode_legacy_event(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        event: &LegacyEventWrapper,
        index: u32,
    ) -> anyhow::Result<Event> {
        let phase = parse_legacy_phase(&event.get_phase()?)?;
        let metadata = get_parsed_metadata(
            block_hash,
            spec_version,
            &self.postgres,
            &self.substrate_client,
            &self.legacy_decode_api_client,
        )
        .await?;
        let pallet_name = &event.event.pallet;
        let pallet = metadata
            .get_pallet_by_name(pallet_name)
            .ok_or(anyhow::Error::msg(format!(
                "Pallet {pallet_name} not found in metadata database."
            )))?;
        let pallet_event_name = &event.event.name;
        let pallet_event =
            pallet
                .get_event_by_name(pallet_event_name)
                .ok_or(anyhow::Error::msg(format!(
                "Event {pallet_event_name} not found in pallet {pallet_name} in metadata database."
            )))?;
        Ok(Event {
            trace_index: None,
            pallet_index: pallet.index,
            pallet_name: pallet_name.clone(),
            pallet_event_index: pallet_event.index,
            pallet_event_name: pallet_event_name.clone(),
            index,
            phase,
            args: event.event.data.clone(),
        })
    }

    async fn get_legacy_events_from_bytes(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        bytes: &mut &[u8],
    ) -> anyhow::Result<Vec<Event>> {
        let legacy_decode_api_client = self.get_legacy_decode_client()?;
        let legacy_events = legacy_decode_api_client
            .decode_events(block_hash, spec_version, bytes)
            .await?;
        let mut events = Vec::new();
        for event in legacy_events.iter() {
            events.push(
                self.decode_legacy_event(block_hash, spec_version, event, events.len() as u32)
                    .await?,
            );
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
                    events.push(decode_metadata_v14_event(
                        metadata_v14,
                        call_type.id,
                        None,
                        pallet_index,
                        pallet_event_index,
                        phase,
                        events.len() as u32,
                        bytes,
                    )?);
                    let _topics = Vec::<sp_core::H256>::decode(bytes)?;
                }
                RuntimeMetadata::V15(metadata_v15) => {
                    events.push(decode_metadata_v15_event(
                        metadata_v15,
                        call_type.id,
                        None,
                        pallet_index,
                        pallet_event_index,
                        phase,
                        events.len() as u32,
                        bytes,
                    )?);
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
        let events = if metadata_version < METADATA_VERSION_LEGACY_THRESHOLD {
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
        let events_key = hex::encode(get_storage_plain_key(
            SYSTEM_PALLET_NAME,
            EVENTS_STORAGE_KEY,
        ));
        let mut processed_events_hex = String::new();
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_index = trace_index as u32;
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key != events_key || trace_data.value.eq_ignore_ascii_case(NONE_VALUE) {
                continue;
            }
            let value =
                extract_trace_value(&trace_data.value, &trace_data.method, &processed_events_hex)?;
            let mut bytes: &[u8] = &hex::decode(&value)?;
            if metadata_version < METADATA_VERSION_LEGACY_THRESHOLD {
                let legacy_decode_api_client = self.get_legacy_decode_client()?;
                let event = legacy_decode_api_client
                    .decode_event(block_hash, spec_version, bytes)
                    .await?;
                events.push(
                    self.decode_legacy_event(block_hash, spec_version, &event, events.len() as u32)
                        .await?,
                );
                continue;
            }
            let call_type = get_runtime_call_type(metadata)?;
            let phase = frame_system::Phase::decode(&mut bytes)?;
            let pallet_index: u8 = Decode::decode(&mut bytes)?;
            let pallet_event_index: u8 = Decode::decode(&mut bytes)?;
            match &metadata {
                RuntimeMetadata::V14(metadata_v14) => {
                    events.push(decode_metadata_v14_event(
                        metadata_v14,
                        call_type.id,
                        Some(trace_index),
                        pallet_index,
                        pallet_event_index,
                        phase,
                        events.len() as u32,
                        &mut bytes,
                    )?);
                }
                RuntimeMetadata::V15(metadata_v15) => {
                    events.push(decode_metadata_v15_event(
                        metadata_v15,
                        call_type.id,
                        Some(trace_index),
                        pallet_index,
                        pallet_event_index,
                        phase,
                        events.len() as u32,
                        &mut bytes,
                    )?);
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
        let metadata = get_parsed_metadata(
            block_hash,
            spec_version,
            &self.postgres,
            &self.substrate_client,
            &self.legacy_decode_api_client,
        )
        .await?;
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

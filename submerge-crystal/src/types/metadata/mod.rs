use convert_case::{Case, Casing};
use frame_metadata::decode_different::DecodeDifferent;
use frame_metadata::v10::RuntimeMetadataV10;
use frame_metadata::v11::RuntimeMetadataV11;
use frame_metadata::v12::RuntimeMetadataV12;
use frame_metadata::v13::RuntimeMetadataV13;
use frame_metadata::v14::RuntimeMetadataV14;
use frame_metadata::v15::RuntimeMetadataV15;
use frame_metadata::v8::RuntimeMetadataV8;
use frame_metadata::v9::RuntimeMetadataV9;
use frame_metadata::RuntimeMetadata;
use paste::paste;
use serde_json::Value as JSONValue;
use util::get_decode_different_string;

use crate::types::metadata::util::{
    get_metadata_version, v14::get_metadata_type_by_id as get_metadata_type_by_id_v14,
    v15::get_metadata_type_by_id as get_metadata_type_by_id_v15,
};

pub mod util;

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub pallets: Vec<MetadataPallet>,
}

impl Metadata {
    pub fn get_pallet_by_index(&self, index: u8) -> Option<&MetadataPallet> {
        self.pallets.iter().find(|pallet| pallet.index == index)
    }

    pub fn get_pallet_by_name(&self, name: &str) -> Option<&MetadataPallet> {
        self.pallets
            .iter()
            .find(|pallet| pallet.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetadataPallet {
    pub id: u32,
    pub index: u8,
    pub name: String,
    pub events: Vec<MetadataEvent>,
    pub constants: Vec<MetadataConstant>,
    pub calls: Vec<MetadataCall>,
    pub storage_items: Vec<MetadataStorageItem>,
    pub errors: Vec<MetadataError>,
}

impl MetadataPallet {
    pub fn get_event_by_index(&self, index: u8) -> Option<&MetadataEvent> {
        self.events.iter().find(|event| event.index == index)
    }

    pub fn get_call_by_index(&self, index: u8) -> Option<&MetadataCall> {
        self.calls.iter().find(|call| call.index == index)
    }

    pub fn get_call_by_name(&self, name: &str) -> Option<&MetadataCall> {
        self.calls
            .iter()
            .find(|call| call.name.eq_ignore_ascii_case(name))
    }

    pub fn get_event_by_name(&self, name: &str) -> Option<&MetadataEvent> {
        self.events
            .iter()
            .find(|event| event.name.eq_ignore_ascii_case(name))
    }

    pub fn get_storage_item_by_name(&self, name: &str) -> Option<&MetadataStorageItem> {
        self.storage_items
            .iter()
            .find(|storage_item| storage_item.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetadataEvent {
    pub id: u32,
    pub index: u8,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataConstant {
    pub id: u32,
    pub index: u8,
    pub name: String,
    pub type_id: Option<u32>,
    pub type_name: String,
    pub value: Vec<u8>,
    pub value_json: Option<JSONValue>,
    pub docs: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataCall {
    pub id: u32,
    pub index: u8,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataStorageItem {
    pub id: u32,
    pub index: u8,
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataError {
    pub id: u32,
    pub index: u8,
    pub name: String,
    pub docs: Vec<String>,
}

fn extract_docs(
    documentation: &DecodeDifferent<&'static [&'static str], Vec<String>>,
) -> Vec<String> {
    match documentation {
        DecodeDifferent::Encode(enc) => enc.iter().map(|s| s.to_string()).collect(),
        DecodeDifferent::Decoded(dec) => dec.clone(),
    }
}

fn extract_module_name(name: &DecodeDifferent<&'static str, String>) -> String {
    match name {
        DecodeDifferent::Encode(name) => name.to_string(),
        DecodeDifferent::Decoded(name) => name.clone(),
    }
}

fn create_base_pallet(id: u32, index: u8, name: String) -> MetadataPallet {
    MetadataPallet {
        id,
        index,
        name: name.to_case(Case::UpperCamel),
        events: Vec::new(),
        constants: Vec::new(),
        calls: Vec::new(),
        storage_items: Vec::new(),
        errors: Vec::new(),
    }
}

macro_rules! from_metadata_version {
    ($version:literal) => {
        impl TryFrom<&paste! {[<RuntimeMetadataV $version>]}> for Metadata {
            type Error = anyhow::Error;

            fn try_from(
                value: &paste! {[<RuntimeMetadataV $version>]},
            ) -> Result<Self, anyhow::Error> {
                let mut metadata = Metadata::default();
                for pallet_metadata in value.pallets.iter() {
                    let mut pallet =
                        create_base_pallet(0, pallet_metadata.index, pallet_metadata.name.clone());
                    // events
                    if let Some(events) = &pallet_metadata.event {
                        let type_id = events.ty.id;
                        let events_type =
                            paste! {[<get_metadata_type_by_id_v $version>](value, type_id)}.ok_or(
                                anyhow::Error::msg(format!(
                                    "Events type not found in pallet {}.",
                                    pallet.name
                                )),
                            )?;
                        match &events_type.ty.type_def {
                            scale_info::TypeDef::Variant(type_def_variant) => {
                                for event_variant in type_def_variant.variants.iter() {
                                    pallet.events.push(MetadataEvent {
                                        id: 0,
                                        index: event_variant.index,
                                        name: event_variant.name.to_case(Case::UpperCamel),
                                        docs: event_variant.docs.clone(),
                                    });
                                }
                            }
                            _ => return Err(anyhow::Error::msg("Non-variant pallet events type.")),
                        }
                    }
                    // constants
                    for (index, constant) in pallet_metadata.constants.iter().enumerate() {
                        let type_id = constant.ty.id;
                        let constant_type =
                            paste! {[<get_metadata_type_by_id_v $version>](value, type_id)}.ok_or(
                                anyhow::Error::msg(format!(
                                    "Constant type with id {type_id} not found in pallet {}.",
                                    pallet.name,
                                )),
                            )?;
                        let type_name = constant_type.ty.path.segments.join("::");
                        pallet.constants.push(MetadataConstant {
                            id: 0,
                            index: index as u8,
                            name: constant.name.to_case(Case::UpperCamel),
                            type_id: Some(constant.ty.id),
                            type_name,
                            value: constant.value.clone(),
                            value_json: None,
                            docs: constant.docs.clone(),
                        });
                    }
                    // calls
                    if let Some(calls) = &pallet_metadata.calls {
                        let type_id = calls.ty.id;
                        let calls_type =
                            paste! {[<get_metadata_type_by_id_v $version>](value, type_id)}.ok_or(
                                anyhow::Error::msg(format!(
                                    "Calls type not found in pallet {}.",
                                    pallet.name
                                )),
                            )?;
                        match &calls_type.ty.type_def {
                            scale_info::TypeDef::Variant(type_def_variant) => {
                                for call_variant in type_def_variant.variants.iter() {
                                    pallet.calls.push(MetadataCall {
                                        id: 0,
                                        index: call_variant.index,
                                        name: call_variant.name.to_case(Case::UpperCamel),
                                        docs: call_variant.docs.clone(),
                                    });
                                }
                            }
                            _ => return Err(anyhow::Error::msg("Non-variant pallet calls type.")),
                        }
                    }
                    // storage items
                    if let Some(storage) = &pallet_metadata.storage {
                        for (index, entry) in storage.entries.iter().enumerate() {
                            pallet.storage_items.push(MetadataStorageItem {
                                id: 0,
                                index: index as u8,
                                name: entry.name.to_case(Case::UpperCamel),
                                docs: entry.docs.clone(),
                            });
                        }
                    }
                    // errors
                    if let Some(error) = &pallet_metadata.error {
                        let type_id = error.ty.id;
                        let errors_type =
                            paste! {[<get_metadata_type_by_id_v $version>](value, type_id)}.ok_or(
                                anyhow::Error::msg(format!(
                                    "Errors type not found in pallet {}.",
                                    pallet.name
                                )),
                            )?;
                        match &errors_type.ty.type_def {
                            scale_info::TypeDef::Variant(type_def_variant) => {
                                for error_variant in type_def_variant.variants.iter() {
                                    pallet.errors.push(MetadataError {
                                        id: 0,
                                        index: error_variant.index,
                                        name: error_variant.name.to_case(Case::UpperCamel),
                                        docs: error_variant.docs.clone(),
                                    });
                                }
                            }
                            _ => return Err(anyhow::Error::msg("Non-variant pallet errors type.")),
                        }
                    }
                    metadata.pallets.push(pallet);
                }
                Ok(metadata)
            }
        }
    };
}

macro_rules! from_legacy_metadata {
    ($a:ty) => {
        #[allow(clippy::too_many_lines)]
        impl From<&$a> for Metadata {
            fn from(value: &$a) -> Self {
                let mut metadata = Metadata::default();
                match &value.modules {
                    DecodeDifferent::Decoded(modules) => {
                        for (index, module) in modules.iter().enumerate() {
                            let name = extract_module_name(&module.name).to_case(Case::UpperCamel);
                            let mut pallet = create_base_pallet(0, index as u8, name);
                            if let Some(module_events) = &module.event {
                                match module_events {
                                    DecodeDifferent::Encode(module_events) => {
                                        for (index, module_event) in
                                            (module_events.0)().iter().enumerate()
                                        {
                                            let name =
                                                get_decode_different_string(&module_event.name)
                                                    .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&module_event.documentation);
                                            pallet.events.push(MetadataEvent {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                    DecodeDifferent::Decoded(module_events) => {
                                        for (index, module_event) in
                                            module_events.iter().enumerate()
                                        {
                                            let name =
                                                get_decode_different_string(&module_event.name)
                                                    .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&module_event.documentation);
                                            pallet.events.push(MetadataEvent {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                }
                            }
                            match &module.constants {
                                DecodeDifferent::Encode(module_constants) => {
                                    for (index, module_constant) in
                                        (module_constants.0)().iter().enumerate()
                                    {
                                        let name =
                                            get_decode_different_string(&module_constant.name)
                                                .to_case(Case::UpperCamel);
                                        let value = match &module_constant.value {
                                            DecodeDifferent::Encode(enc) => enc.0.default_byte(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let type_name = match &module_constant.ty {
                                            DecodeDifferent::Encode(enc) => enc.to_string(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let docs: Vec<String> =
                                            extract_docs(&module_constant.documentation);
                                        pallet.constants.push(MetadataConstant {
                                            id: 0,
                                            index: index as u8,
                                            name,
                                            type_id: None,
                                            type_name,
                                            value,
                                            value_json: None,
                                            docs,
                                        });
                                    }
                                }
                                DecodeDifferent::Decoded(module_constants) => {
                                    for (index, module_constant) in
                                        module_constants.iter().enumerate()
                                    {
                                        let name =
                                            get_decode_different_string(&module_constant.name)
                                                .to_case(Case::UpperCamel);
                                        let value = match &module_constant.value {
                                            DecodeDifferent::Encode(enc) => enc.0.default_byte(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let type_name = match &module_constant.ty {
                                            DecodeDifferent::Encode(enc) => enc.to_string(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let docs: Vec<String> =
                                            extract_docs(&module_constant.documentation);
                                        pallet.constants.push(MetadataConstant {
                                            id: 0,
                                            index: index as u8,
                                            name,
                                            type_id: None,
                                            type_name,
                                            value,
                                            value_json: None,
                                            docs,
                                        });
                                    }
                                }
                            }
                            if let Some(calls) = &module.calls {
                                match calls {
                                    DecodeDifferent::Encode(calls) => {
                                        for (index, call) in calls.0().iter().enumerate() {
                                            let name = get_decode_different_string(&call.name)
                                                .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&call.documentation);
                                            pallet.calls.push(MetadataCall {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                    DecodeDifferent::Decoded(calls) => {
                                        for (index, call) in calls.iter().enumerate() {
                                            let name = get_decode_different_string(&call.name)
                                                .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&call.documentation);
                                            pallet.calls.push(MetadataCall {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                }
                            }
                            if let Some(storage) = &module.storage {
                                match &storage {
                                    DecodeDifferent::Encode(storage) => {
                                        match &storage.0().entries {
                                            DecodeDifferent::Encode(entries) => {
                                                for (index, entry) in entries.iter().enumerate() {
                                                    let name =
                                                        get_decode_different_string(&entry.name)
                                                            .to_case(Case::UpperCamel);
                                                    let docs: Vec<String> =
                                                        extract_docs(&entry.documentation);
                                                    pallet.storage_items.push(
                                                        MetadataStorageItem {
                                                            id: 0,
                                                            index: index as u8,
                                                            name,
                                                            docs,
                                                        },
                                                    );
                                                }
                                            }
                                            DecodeDifferent::Decoded(entries) => {
                                                for (index, entry) in entries.iter().enumerate() {
                                                    let name =
                                                        get_decode_different_string(&entry.name)
                                                            .to_case(Case::UpperCamel);
                                                    let docs: Vec<String> =
                                                        extract_docs(&entry.documentation);
                                                    pallet.storage_items.push(
                                                        MetadataStorageItem {
                                                            id: 0,
                                                            index: index as u8,
                                                            name,
                                                            docs,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    DecodeDifferent::Decoded(storage) => match &storage.entries {
                                        DecodeDifferent::Encode(entries) => {
                                            for (index, entry) in entries.iter().enumerate() {
                                                let name = get_decode_different_string(&entry.name)
                                                    .to_case(Case::UpperCamel);
                                                let docs: Vec<String> =
                                                    extract_docs(&entry.documentation);
                                                pallet.storage_items.push(MetadataStorageItem {
                                                    id: 0,
                                                    index: index as u8,
                                                    name,
                                                    docs,
                                                });
                                            }
                                        }
                                        DecodeDifferent::Decoded(entries) => {
                                            for (index, entry) in entries.iter().enumerate() {
                                                let name = get_decode_different_string(&entry.name)
                                                    .to_case(Case::UpperCamel);
                                                let docs: Vec<String> =
                                                    extract_docs(&entry.documentation);
                                                pallet.storage_items.push(MetadataStorageItem {
                                                    id: 0,
                                                    index: index as u8,
                                                    name,
                                                    docs,
                                                });
                                            }
                                        }
                                    },
                                }
                            }
                            match &module.errors {
                                DecodeDifferent::Encode(errors) => {
                                    for (index, error) in errors.0().iter().enumerate() {
                                        let name = get_decode_different_string(&error.name)
                                            .to_case(Case::UpperCamel);
                                        let docs: Vec<String> = extract_docs(&error.documentation);
                                        pallet.errors.push(MetadataError {
                                            id: 0,
                                            index: index as u8,
                                            name,
                                            docs,
                                        });
                                    }
                                }
                                DecodeDifferent::Decoded(errors) => {
                                    for (index, error) in errors.iter().enumerate() {
                                        let name = get_decode_different_string(&error.name)
                                            .to_case(Case::UpperCamel);
                                        let docs: Vec<String> = extract_docs(&error.documentation);
                                        pallet.errors.push(MetadataError {
                                            id: 0,
                                            index: index as u8,
                                            name,
                                            docs,
                                        });
                                    }
                                }
                            }
                            metadata.pallets.push(pallet);
                        }
                    }
                    DecodeDifferent::Encode(modules) => {
                        for (index, module) in modules.iter().enumerate() {
                            let name = match &module.name {
                                DecodeDifferent::Encode(name) => name.to_string(),
                                DecodeDifferent::Decoded(name) => name.clone(),
                            }
                            .to_case(Case::UpperCamel);
                            let mut pallet = MetadataPallet {
                                id: 0,
                                index: index as u8,
                                name,
                                events: Vec::new(),
                                constants: Vec::new(),
                                calls: Vec::new(),
                                storage_items: Vec::new(),
                                errors: Vec::new(),
                            };
                            if let Some(module_events) = &module.event {
                                match module_events {
                                    DecodeDifferent::Encode(module_events) => {
                                        for (index, module_event) in
                                            (module_events.0)().iter().enumerate()
                                        {
                                            let name = match &module_event.name {
                                                DecodeDifferent::Encode(name) => name.to_string(),
                                                DecodeDifferent::Decoded(name) => name.clone(),
                                            }
                                            .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&module_event.documentation);
                                            pallet.events.push(MetadataEvent {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                    DecodeDifferent::Decoded(module_events) => {
                                        for (index, module_event) in
                                            module_events.iter().enumerate()
                                        {
                                            let name = match &module_event.name {
                                                DecodeDifferent::Encode(name) => name.to_string(),
                                                DecodeDifferent::Decoded(name) => name.clone(),
                                            }
                                            .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&module_event.documentation);
                                            pallet.events.push(MetadataEvent {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                }
                            }
                            match &module.constants {
                                DecodeDifferent::Encode(module_constants) => {
                                    for (index, module_constant) in
                                        (module_constants.0)().iter().enumerate()
                                    {
                                        let name =
                                            get_decode_different_string(&module_constant.name)
                                                .to_case(Case::UpperCamel);
                                        let value = match &module_constant.value {
                                            DecodeDifferent::Encode(enc) => enc.0.default_byte(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let type_name = match &module_constant.ty {
                                            DecodeDifferent::Encode(enc) => enc.to_string(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let docs: Vec<String> =
                                            extract_docs(&module_constant.documentation);
                                        pallet.constants.push(MetadataConstant {
                                            id: 0,
                                            index: index as u8,
                                            name,
                                            type_id: None,
                                            type_name,
                                            value,
                                            value_json: None,
                                            docs,
                                        });
                                    }
                                }
                                DecodeDifferent::Decoded(module_constants) => {
                                    for (index, module_constant) in
                                        module_constants.iter().enumerate()
                                    {
                                        let name =
                                            get_decode_different_string(&module_constant.name)
                                                .to_case(Case::UpperCamel);
                                        let value = match &module_constant.value {
                                            DecodeDifferent::Encode(enc) => enc.0.default_byte(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let type_name = match &module_constant.ty {
                                            DecodeDifferent::Encode(enc) => enc.to_string(),
                                            DecodeDifferent::Decoded(dec) => dec.clone(),
                                        };
                                        let docs: Vec<String> =
                                            extract_docs(&module_constant.documentation);
                                        pallet.constants.push(MetadataConstant {
                                            id: 0,
                                            index: index as u8,
                                            name,
                                            type_id: None,
                                            type_name,
                                            value,
                                            value_json: None,
                                            docs,
                                        });
                                    }
                                }
                            }
                            if let Some(calls) = &module.calls {
                                match calls {
                                    DecodeDifferent::Encode(calls) => {
                                        for (index, call) in calls.0().iter().enumerate() {
                                            let name = get_decode_different_string(&call.name)
                                                .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&call.documentation);
                                            pallet.calls.push(MetadataCall {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                    DecodeDifferent::Decoded(calls) => {
                                        for (index, call) in calls.iter().enumerate() {
                                            let name = get_decode_different_string(&call.name)
                                                .to_case(Case::UpperCamel);
                                            let docs: Vec<String> =
                                                extract_docs(&call.documentation);
                                            pallet.calls.push(MetadataCall {
                                                id: 0,
                                                index: index as u8,
                                                name,
                                                docs,
                                            });
                                        }
                                    }
                                }
                            }
                            metadata.pallets.push(pallet);
                        }
                    }
                }
                metadata
            }
        }
    };
}

from_legacy_metadata!(RuntimeMetadataV8);
from_legacy_metadata!(RuntimeMetadataV9);
from_legacy_metadata!(RuntimeMetadataV10);
from_legacy_metadata!(RuntimeMetadataV11);
from_legacy_metadata!(RuntimeMetadataV12);
from_legacy_metadata!(RuntimeMetadataV13);

from_metadata_version!(14);
from_metadata_version!(15);

impl TryFrom<&RuntimeMetadata> for Metadata {
    type Error = anyhow::Error;

    fn try_from(metadata: &RuntimeMetadata) -> Result<Self, anyhow::Error> {
        match metadata {
            RuntimeMetadata::V8(runtime_metadata_v8) => Ok(runtime_metadata_v8.into()),
            RuntimeMetadata::V9(runtime_metadata_v9) => Ok(runtime_metadata_v9.into()),
            RuntimeMetadata::V10(runtime_metadata_v10) => Ok(runtime_metadata_v10.into()),
            RuntimeMetadata::V11(runtime_metadata_v11) => Ok(runtime_metadata_v11.into()),
            RuntimeMetadata::V12(runtime_metadata_v12) => Ok(runtime_metadata_v12.into()),
            RuntimeMetadata::V13(runtime_metadata_v13) => Ok(runtime_metadata_v13.into()),
            RuntimeMetadata::V14(runtime_metadata_v14) => runtime_metadata_v14.try_into(),
            _ => {
                let version = get_metadata_version(metadata);
                anyhow::bail!("Unsupported metadata version {version}.");
            }
        }
    }
}

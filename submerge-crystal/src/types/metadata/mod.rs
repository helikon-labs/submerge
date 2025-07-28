use convert_case::{Case, Casing};
use frame_metadata::decode_different::DecodeDifferent;
use frame_metadata::v10::RuntimeMetadataV10;
use frame_metadata::v11::RuntimeMetadataV11;
use frame_metadata::v12::RuntimeMetadataV12;
use frame_metadata::v13::RuntimeMetadataV13;
use frame_metadata::v14::RuntimeMetadataV14;
use frame_metadata::v15::RuntimeMetadataV15;
use frame_metadata::v16::RuntimeMetadataV16;
use frame_metadata::v8::RuntimeMetadataV8;
use frame_metadata::v9::RuntimeMetadataV9;
use frame_metadata::RuntimeMetadata;
use util::get_decode_different_string;

pub mod util;

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub pallets: Vec<MetadataPallet>,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataPallet {
    pub index: u8,
    pub name: String,
    pub events: Vec<MetadataPalletEvent>,
    pub constants: Vec<MetadataPalletConstant>,
    pub calls: Vec<MetadataPalletCall>,
    pub storage_items: Vec<MetadataPalletStorageItem>,
    pub errors: Vec<MetadataPalletError>,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataPalletEvent {
    pub index: u8,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataPalletConstant {
    pub index: u8,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataPalletCall {
    pub index: u8,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataPalletStorageItem {
    pub index: u8,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct MetadataPalletError {
    pub index: u8,
    pub name: String,
}

macro_rules! from_metadata {
    ($a:ty) => {
        impl TryFrom<&$a> for Metadata {
            type Error = anyhow::Error;

            fn try_from(value: &$a) -> Result<Self, anyhow::Error> {
                let mut metadata = Metadata::default();
                for pallet_metadata in value.pallets.iter() {
                    let mut pallet = MetadataPallet {
                        index: pallet_metadata.index,
                        name: pallet_metadata.name.to_case(Case::UpperCamel),
                        events: Vec::new(),
                        constants: Vec::new(),
                        calls: Vec::new(),
                        storage_items: Vec::new(),
                        errors: Vec::new(),
                    };
                    // events
                    if let Some(events) = &pallet_metadata.event {
                        let events_type = value
                            .types
                            .types
                            .iter()
                            .find(|metadata_type| metadata_type.id == events.ty.id)
                            .ok_or(anyhow::Error::msg(format!(
                                "Events type not found in pallet {}.",
                                pallet.name
                            )))?;
                        match &events_type.ty.type_def {
                            scale_info::TypeDef::Variant(type_def_variant) => {
                                for event_variant in type_def_variant.variants.iter() {
                                    pallet.events.push(MetadataPalletEvent {
                                        index: event_variant.index,
                                        name: event_variant.name.to_case(Case::UpperCamel),
                                    });
                                }
                            }
                            _ => return Err(anyhow::Error::msg("Non-variant pallet events type.")),
                        }
                    }
                    // constants
                    for (index, constant) in pallet_metadata.constants.iter().enumerate() {
                        pallet.constants.push(MetadataPalletConstant {
                            index: index as u8,
                            name: constant.name.to_case(Case::UpperCamel),
                        });
                    }
                    // calls
                    if let Some(calls) = &pallet_metadata.calls {
                        let calls_type = value
                            .types
                            .types
                            .iter()
                            .find(|metadata_type| metadata_type.id == calls.ty.id)
                            .ok_or(anyhow::Error::msg(format!(
                                "Calls type not found in pallet {}.",
                                pallet.name
                            )))?;
                        match &calls_type.ty.type_def {
                            scale_info::TypeDef::Variant(type_def_variant) => {
                                for call_variant in type_def_variant.variants.iter() {
                                    pallet.calls.push(MetadataPalletCall {
                                        index: call_variant.index,
                                        name: call_variant.name.to_case(Case::UpperCamel),
                                    });
                                }
                            }
                            _ => return Err(anyhow::Error::msg("Non-variant pallet calls type.")),
                        }
                    }
                    // storage items
                    if let Some(storage) = &pallet_metadata.storage {
                        for (index, entry) in storage.entries.iter().enumerate() {
                            pallet.storage_items.push(MetadataPalletStorageItem {
                                index: index as u8,
                                name: entry.name.to_case(Case::UpperCamel),
                            });
                        }
                    }
                    // errors
                    if let Some(error) = &pallet_metadata.error {
                        let errors_type = value
                            .types
                            .types
                            .iter()
                            .find(|metadata_type| metadata_type.id == error.ty.id)
                            .ok_or(anyhow::Error::msg(format!(
                                "Calls type not found in pallet {}.",
                                pallet.name
                            )))?;
                        match &errors_type.ty.type_def {
                            scale_info::TypeDef::Variant(type_def_variant) => {
                                for error_variant in type_def_variant.variants.iter() {
                                    pallet.errors.push(MetadataPalletError {
                                        index: error_variant.index,
                                        name: error_variant.name.to_case(Case::UpperCamel),
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
        impl From<&$a> for Metadata {
            #[allow(clippy::cognitive_complexity)]
            fn from(value: &$a) -> Self {
                let mut metadata = Metadata::default();
                match &value.modules {
                    DecodeDifferent::Decoded(modules) => {
                        for (index, module) in modules.iter().enumerate() {
                            let name = match &module.name {
                                DecodeDifferent::Encode(name) => name.to_string(),
                                DecodeDifferent::Decoded(name) => name.clone(),
                            }
                            .to_case(Case::UpperCamel);
                            let mut pallet = MetadataPallet {
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
                                            let name =
                                                get_decode_different_string(&module_event.name)
                                                    .to_case(Case::UpperCamel);
                                            pallet.events.push(MetadataPalletEvent {
                                                index: index as u8,
                                                name,
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
                                            pallet.events.push(MetadataPalletEvent {
                                                index: index as u8,
                                                name,
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
                                        pallet.constants.push(MetadataPalletConstant {
                                            index: index as u8,
                                            name,
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
                                        pallet.constants.push(MetadataPalletConstant {
                                            index: index as u8,
                                            name,
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
                                            pallet.calls.push(MetadataPalletCall {
                                                index: index as u8,
                                                name,
                                            });
                                        }
                                    }
                                    DecodeDifferent::Decoded(calls) => {
                                        for (index, call) in calls.iter().enumerate() {
                                            let name = get_decode_different_string(&call.name)
                                                .to_case(Case::UpperCamel);
                                            pallet.calls.push(MetadataPalletCall {
                                                index: index as u8,
                                                name,
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
                                                    pallet.storage_items.push(
                                                        MetadataPalletStorageItem {
                                                            index: index as u8,
                                                            name,
                                                        },
                                                    );
                                                }
                                            }
                                            DecodeDifferent::Decoded(entries) => {
                                                for (index, entry) in entries.iter().enumerate() {
                                                    let name =
                                                        get_decode_different_string(&entry.name)
                                                            .to_case(Case::UpperCamel);
                                                    pallet.storage_items.push(
                                                        MetadataPalletStorageItem {
                                                            index: index as u8,
                                                            name,
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
                                                pallet.storage_items.push(
                                                    MetadataPalletStorageItem {
                                                        index: index as u8,
                                                        name,
                                                    },
                                                );
                                            }
                                        }
                                        DecodeDifferent::Decoded(entries) => {
                                            for (index, entry) in entries.iter().enumerate() {
                                                let name = get_decode_different_string(&entry.name)
                                                    .to_case(Case::UpperCamel);
                                                pallet.storage_items.push(
                                                    MetadataPalletStorageItem {
                                                        index: index as u8,
                                                        name,
                                                    },
                                                );
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
                                        pallet.errors.push(MetadataPalletError {
                                            index: index as u8,
                                            name,
                                        });
                                    }
                                }
                                DecodeDifferent::Decoded(errors) => {
                                    for (index, error) in errors.iter().enumerate() {
                                        let name = get_decode_different_string(&error.name)
                                            .to_case(Case::UpperCamel);
                                        pallet.errors.push(MetadataPalletError {
                                            index: index as u8,
                                            name,
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
                                            pallet.events.push(MetadataPalletEvent {
                                                index: index as u8,
                                                name,
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
                                            pallet.events.push(MetadataPalletEvent {
                                                index: index as u8,
                                                name,
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
                                        pallet.constants.push(MetadataPalletConstant {
                                            index: index as u8,
                                            name,
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
                                        pallet.constants.push(MetadataPalletConstant {
                                            index: index as u8,
                                            name,
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
                                            pallet.calls.push(MetadataPalletCall {
                                                index: index as u8,
                                                name,
                                            });
                                        }
                                    }
                                    DecodeDifferent::Decoded(calls) => {
                                        for (index, call) in calls.iter().enumerate() {
                                            let name = get_decode_different_string(&call.name)
                                                .to_case(Case::UpperCamel);
                                            pallet.calls.push(MetadataPalletCall {
                                                index: index as u8,
                                                name,
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

from_metadata!(RuntimeMetadataV14);
from_metadata!(RuntimeMetadataV15);
from_metadata!(RuntimeMetadataV16);

impl TryFrom<&RuntimeMetadata> for Metadata {
    type Error = anyhow::Error;

    fn try_from(runtime_metadata: &RuntimeMetadata) -> Result<Self, anyhow::Error> {
        match runtime_metadata {
            RuntimeMetadata::V8(runtime_metadata_v8) => Ok(runtime_metadata_v8.into()),
            RuntimeMetadata::V9(runtime_metadata_v9) => Ok(runtime_metadata_v9.into()),
            RuntimeMetadata::V10(runtime_metadata_v10) => Ok(runtime_metadata_v10.into()),
            RuntimeMetadata::V11(runtime_metadata_v11) => Ok(runtime_metadata_v11.into()),
            RuntimeMetadata::V12(runtime_metadata_v12) => Ok(runtime_metadata_v12.into()),
            RuntimeMetadata::V13(runtime_metadata_v13) => Ok(runtime_metadata_v13.into()),
            RuntimeMetadata::V14(runtime_metadata_v14) => runtime_metadata_v14.try_into(),
            RuntimeMetadata::V15(runtime_metadata_v15) => runtime_metadata_v15.try_into(),
            RuntimeMetadata::V16(runtime_metadata_v16) => runtime_metadata_v16.try_into(),
            _ => anyhow::bail!("Unsupported metadata version <8."),
        }
    }
}

use convert_case::{Case, Casing};
use frame_metadata::RuntimeMetadata;
use serde_json::Value;
use std::marker::PhantomData;

pub(crate) fn get_metadata_version(metadata: &RuntimeMetadata) -> u32 {
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

pub(crate) struct JsonValueVisitor<R>(PhantomData<R>);

impl<R> JsonValueVisitor<R> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R: scale_decode::TypeResolver> scale_decode::visitor::Visitor for JsonValueVisitor<R> {
    type Value<'scale, 'resolver> = Value;
    type Error = scale_decode::visitor::DecodeError;
    type TypeResolver = R;

    fn visit_bool<'scale, 'resolver>(
        self,
        value: bool,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::Bool(value))
    }

    fn visit_char<'scale, 'resolver>(
        self,
        value: char,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u8<'scale, 'resolver>(
        self,
        value: u8,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u16<'scale, 'resolver>(
        self,
        value: u16,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u32<'scale, 'resolver>(
        self,
        value: u32,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u64<'scale, 'resolver>(
        self,
        value: u64,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u128<'scale, 'resolver>(
        self,
        value: u128,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u256<'resolver>(
        self,
        value: &[u8; 32],
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
        Ok(Value::String(format!("0x{}", hex::encode(value))))
    }

    fn visit_i8<'scale, 'resolver>(
        self,
        value: i8,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i16<'scale, 'resolver>(
        self,
        value: i16,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i32<'scale, 'resolver>(
        self,
        value: i32,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i64<'scale, 'resolver>(
        self,
        value: i64,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i128<'scale, 'resolver>(
        self,
        value: i128,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i256<'resolver>(
        self,
        value: &[u8; 32],
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
        Ok(Value::String(format!("0x{}", hex::encode(value))))
    }

    fn visit_sequence<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Sequence<'scale, 'resolver, Self::TypeResolver>,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        // Check if this is a sequence of u8 values
        let mut vals = vec![];
        let mut u8_bytes = vec![];
        let mut is_u8_sequence = true;

        while let Some(val) = value.decode_item(JsonValueVisitor::new()) {
            let val = val?;
            if let Value::String(s) = &val {
                if let Ok(byte) = s.parse::<u8>() {
                    u8_bytes.push(byte);
                } else {
                    is_u8_sequence = false;
                }
            } else {
                is_u8_sequence = false;
            }
            vals.push(val);
        }

        if is_u8_sequence && !u8_bytes.is_empty() {
            Ok(Value::String(format!("0x{}", hex::encode(&u8_bytes))))
        } else {
            Ok(Value::Array(vals))
        }
    }

    fn visit_composite<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Composite<'scale, 'resolver, Self::TypeResolver>,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        let mut field_map = serde_json::Map::new();
        for field in value.by_ref() {
            let field = field?;
            let field_value = field.decode_with_visitor(JsonValueVisitor::new())?;
            let field_name = field.name().unwrap_or("").to_owned();
            field_map.insert(field_name.to_case(Case::Camel), field_value);
        }
        if field_map.len() == 1 && field_map.keys().all(|field| field.is_empty()) {
            Ok(field_map.get("").unwrap().clone())
        } else {
            Ok(Value::Object(field_map))
        }
    }

    fn visit_tuple<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Tuple<'scale, 'resolver, Self::TypeResolver>,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(JsonValueVisitor::new()) {
            let val = val?;
            vals.push(val);
        }
        Ok(Value::Array(vals))
    }

    fn visit_str<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Str<'scale>,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.as_str()?.to_owned()))
    }

    fn visit_variant<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Variant<'scale, 'resolver, Self::TypeResolver>,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        if value.name() == "None" {
            return Ok(Value::Null);
        }
        let mut field_map = serde_json::Map::new();
        let mut has_named_fields = false;

        for field in value.fields().by_ref() {
            let field = field?;
            let field_value = field.decode_with_visitor(JsonValueVisitor::new())?;

            if let Some(field_name) = field.name() {
                field_map.insert(field_name.to_case(Case::Camel), field_value);
                has_named_fields = true;
            } else {
                field_map.insert(format!("field_{}", field_map.len()), field_value);
            }
        }

        if has_named_fields {
            Ok(Value::Object(field_map))
        } else {
            let values: Vec<Value> = field_map.values().cloned().collect();
            let mut result = serde_json::Map::new();
            result.insert("type".to_string(), Value::String(value.name().to_string()));
            result.insert(
                "value".to_string(),
                if values.len() == 1 {
                    values.into_iter().next().unwrap()
                } else {
                    Value::Array(values)
                },
            );
            Ok(Value::Object(result))
        }
    }

    fn visit_array<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Array<'scale, 'resolver, Self::TypeResolver>,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        // Check if this is an array of u8 values
        let mut vals = vec![];
        let mut u8_bytes = vec![];
        let mut is_u8_array = true;

        while let Some(val) = value.decode_item(JsonValueVisitor::new()) {
            let val = val?;
            if let Value::String(s) = &val {
                if let Ok(byte) = s.parse::<u8>() {
                    u8_bytes.push(byte);
                } else {
                    is_u8_array = false;
                }
            } else {
                is_u8_array = false;
            }
            vals.push(val);
        }

        if is_u8_array && !u8_bytes.is_empty() {
            Ok(Value::String(format!("0x{}", hex::encode(&u8_bytes))))
        } else {
            Ok(Value::Array(vals))
        }
    }

    fn visit_bitsequence<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::BitSequence<'scale>,
        _type_id: scale_decode::visitor::TypeIdFor<Self>,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        let bit_vec: Result<Vec<bool>, _> = value.decode()?.collect();
        let bit_vec = bit_vec?;
        let mut bytes = Vec::new();
        for chunk in bit_vec.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    match value.format.order {
                        scale_type_resolver::BitsOrderFormat::Lsb0 => byte |= 1 << (7 - i),
                        scale_type_resolver::BitsOrderFormat::Msb0 => byte |= 1 << i,
                    }
                }
            }
            bytes.push(byte);
        }
        Ok(Value::String(format!("0x{}", hex::encode(&bytes))))
    }
}

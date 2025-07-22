use convert_case::{Case, Casing};
use rustc_hash::FxHashMap as HashMap;
use scale_info::PortableRegistry;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug)]
pub struct Call {
    pub pallet_index: u8,
    pub pallet_name: String,
    pub pallet_call_index: u8,
    pub pallet_call_name: String,
    pub args: Value,
}

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Call(Box<Call>),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Box<Value>>),
}

impl From<Value> for JsonValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => JsonValue::Null,
            Value::Call(call) => {
                let mut json_map: JsonMap<String, JsonValue> = JsonMap::new();
                json_map.insert(
                    "palletIndex".to_string(),
                    JsonValue::String(call.pallet_index.to_string()),
                );
                json_map.insert(
                    "palletName".to_string(),
                    JsonValue::String(call.pallet_name.clone()),
                );
                json_map.insert(
                    "callIndex".to_string(),
                    JsonValue::String(call.pallet_call_index.to_string()),
                );
                json_map.insert(
                    "callName".to_string(),
                    JsonValue::String(call.pallet_call_name.clone()),
                );
                json_map.insert("args".to_string(), call.args.into());
                JsonValue::Object(json_map)
            }
            Value::Bool(value) => JsonValue::Bool(value),
            Value::String(value) => JsonValue::String(value),
            Value::Array(values) => {
                JsonValue::Array(values.iter().cloned().map(|v| v.into()).collect())
            }
            Value::Object(map) => {
                let mut json_map: JsonMap<String, JsonValue> = JsonMap::new();
                for (key, value) in map.iter() {
                    let value = &**value;
                    json_map.insert(key.clone(), value.clone().into());
                }
                JsonValue::Object(json_map)
            }
        }
    }
}

pub struct ValueVisitor {
    call_type_id: u32,
    call_pallet: Option<(u8, String)>,
}

impl ValueVisitor {
    pub fn new(call_type_id: u32, call_pallet: Option<(u8, String)>) -> Self {
        Self {
            call_type_id,
            call_pallet,
        }
    }
}

impl scale_decode::visitor::Visitor for ValueVisitor {
    type Value<'scale, 'resolver> = Value;
    type Error = anyhow::Error;
    type TypeResolver = PortableRegistry;

    fn visit_bool<'scale, 'resolver>(
        self,
        value: bool,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::Bool(value))
    }

    fn visit_char<'scale, 'resolver>(
        self,
        value: char,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u8<'scale, 'resolver>(
        self,
        value: u8,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u16<'scale, 'resolver>(
        self,
        value: u16,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u32<'scale, 'resolver>(
        self,
        value: u32,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u64<'scale, 'resolver>(
        self,
        value: u64,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u128<'scale, 'resolver>(
        self,
        value: u128,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_u256<'resolver>(
        self,
        value: &[u8; 32],
        _type_id: u32,
    ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
        Ok(Value::String(format!("0x{}", hex::encode(value))))
    }

    fn visit_i8<'scale, 'resolver>(
        self,
        value: i8,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i16<'scale, 'resolver>(
        self,
        value: i16,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i32<'scale, 'resolver>(
        self,
        value: i32,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i64<'scale, 'resolver>(
        self,
        value: i64,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i128<'scale, 'resolver>(
        self,
        value: i128,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn visit_i256<'resolver>(
        self,
        value: &[u8; 32],
        _type_id: u32,
    ) -> Result<Self::Value<'_, 'resolver>, Self::Error> {
        Ok(Value::String(format!("0x{}", hex::encode(value))))
    }

    fn visit_sequence<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Sequence<'scale, 'resolver, Self::TypeResolver>,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        // Check if this is a sequence of u8 values
        let mut vals = vec![];
        let mut u8_bytes = vec![];
        let mut is_u8_sequence = true;

        while let Some(val) = value.decode_item(ValueVisitor::new(self.call_type_id, None)) {
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
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        let mut field_map = HashMap::default();
        for field in value.by_ref() {
            let field = field?;
            let field_value =
                field.decode_with_visitor(ValueVisitor::new(self.call_type_id, None))?;
            let field_name = field.name().unwrap_or("").to_owned();
            field_map.insert(field_name.to_case(Case::Camel), Box::new(field_value));
        }
        if field_map.len() == 1 && field_map.keys().all(|field| field.is_empty()) {
            Ok(*field_map.get("").unwrap().clone())
        } else {
            Ok(Value::Object(field_map))
        }
    }

    fn visit_tuple<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Tuple<'scale, 'resolver, Self::TypeResolver>,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        let mut vals = vec![];
        while let Some(val) = value.decode_item(ValueVisitor::new(self.call_type_id, None)) {
            let val = val?;
            vals.push(val);
        }
        Ok(Value::Array(vals))
    }

    fn visit_str<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Str<'scale>,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(Value::String(value.as_str()?.to_owned()))
    }

    fn visit_variant<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Variant<'scale, 'resolver, Self::TypeResolver>,
        type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        let name = value.name().to_owned();
        if name.to_lowercase() == "none" {
            return Ok(Value::Null);
        }

        if type_id == self.call_type_id {
            for field in value.fields().by_ref() {
                let field = field?;
                let field_value = field.decode_with_visitor(ValueVisitor::new(
                    self.call_type_id,
                    Some((value.index(), name.clone())),
                ))?;
                match field_value {
                    Value::Null => todo!(),
                    Value::Call(_) => return Ok(field_value.clone()),
                    _ => anyhow::bail!("Call field cannot have any type other than a call."),
                }
            }
            anyhow::bail!("Call field could not be decoded.");
        }

        let mut field_map = HashMap::default();
        let mut has_named_fields = false;
        for field in value.fields().by_ref() {
            let field = field?;
            let field_value =
                field.decode_with_visitor(ValueVisitor::new(self.call_type_id, None))?;
            if let Some(field_name) = field.name() {
                field_map.insert(field_name.to_case(Case::Camel), Box::new(field_value));
                has_named_fields = true;
            } else {
                field_map.insert(format!("field_{}", field_map.len()), Box::new(field_value));
            }
        }

        if let Some(call_pallet) = self.call_pallet {
            let pallet_index = call_pallet.0;
            let pallet_name = call_pallet.1.clone();
            let pallet_call_name = name.to_case(Case::UpperCamel);
            let pallet_call_index = value.index();
            Ok(Value::Call(Box::new(Call {
                pallet_index,
                pallet_name,
                pallet_call_index,
                pallet_call_name,
                args: Value::Object(field_map),
            })))
        } else if has_named_fields {
            Ok(Value::Object(field_map))
        } else {
            let values: Vec<Box<Value>> = field_map.values().cloned().collect();
            let mut result = HashMap::default();
            result.insert(
                "type".to_string(),
                Box::new(Value::String(value.name().to_string())),
            );
            result.insert(
                "value".to_string(),
                if values.len() == 1 {
                    values.into_iter().next().unwrap()
                } else {
                    Box::new(Value::Array(values.iter().map(|v| *v.clone()).collect()))
                },
            );
            Ok(Value::Object(result))
        }
    }

    fn visit_array<'scale, 'resolver>(
        self,
        value: &mut scale_decode::visitor::types::Array<'scale, 'resolver, Self::TypeResolver>,
        _type_id: u32,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        let mut vals = vec![];
        let mut u8_bytes = vec![];
        let mut is_u8_array = true;

        while let Some(val) = value.decode_item(ValueVisitor::new(self.call_type_id, None)) {
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
        _type_id: u32,
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

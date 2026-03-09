use chrono::{DateTime, NaiveDateTime};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value as JSONValue;

pub fn float_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let f: f64 = Deserialize::deserialize(deserializer)?;
    Ok(f.to_string())
}

pub fn iso_8601_to_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let string: String = Deserialize::deserialize(deserializer)?;
    let date_time = DateTime::parse_from_rfc3339(&string).map_err(serde::de::Error::custom)?;
    Ok(date_time.naive_utc()) // Convert to NaiveDateTime
}

pub fn iso_8601_to_optional_naive_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_string: Option<String> = Option::deserialize(deserializer)?;

    if let Some(string) = opt_string {
        let date_time = DateTime::parse_from_rfc3339(&string).map_err(serde::de::Error::custom)?;
        Ok(Some(date_time.naive_utc())) // Convert to NaiveDateTime
    } else {
        Ok(None) // Handle missing or null value
    }
}

pub fn deserialize_empty_object_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned, // Use DeserializeOwned instead of Deserialize<'de>
{
    let value: JSONValue = Deserialize::deserialize(deserializer)?;
    if let Some(value) = value.as_object() {
        if value.is_empty() {
            return Ok(None); // Convert empty `{}` to None
        }
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(serde::de::Error::custom)
}

pub fn strip_nuls(value: &mut JSONValue) {
    match value {
        JSONValue::String(str) if str.contains('\0') => {
            // often these are right-padded; trim_end_matches is safest
            *str = str.replace('\0', "");
        }
        JSONValue::Array(array) => array.iter_mut().for_each(strip_nuls),
        JSONValue::Object(map) => map.values_mut().for_each(strip_nuls),
        _ => {}
    }
}

pub mod hex_serde {
    use serde::Serializer;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
    }
}

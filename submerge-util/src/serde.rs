use chrono::{DateTime, NaiveDateTime};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

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
    let value: Value = Deserialize::deserialize(deserializer)?;
    if value.is_object() && value.as_object().unwrap().is_empty() {
        Ok(None) // Convert empty `{}` to None
    } else {
        serde_json::from_value(value)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

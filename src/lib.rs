pub mod device;
pub mod hub;
pub mod scene;

pub use device::{Device, DeviceData, DeviceType};
pub use scene::Scene;

use serde::Deserialize;

pub(crate) fn deserialize_datetime<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str = String::deserialize(deserializer)?;
    match date_str.parse() {
        Ok(system_time) => Ok(system_time),
        Err(_) => Err(serde::de::Error::custom("Invalid date format")),
    }
}

pub(crate) fn deserialize_datetime_optional<'de, D>(
    deserializer: D,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match deserialize_datetime(deserializer) {
        Ok(system_time) => Ok(Some(system_time)),
        Err(_) => Ok(None),
    }
}

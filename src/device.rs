use crate::deserialize_datetime;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Device {
    Blind(DeviceData),
    Controller(DeviceData),
    Gateway(DeviceData),
    Light(DeviceData),
    Outlet(DeviceData),
    Sensor(DeviceData),
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct DeviceData {
    pub id: String,
    pub device_type: DeviceType,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_reachable: bool,
    pub is_hidden: Option<bool>,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub room: Option<Room>,
    pub attributes: Attributes,
    pub remote_links: Vec<String>,
    pub capabilities: Capabilities,
    // TODO: What is this?
    // pub device_set: Vec<?>,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub can_send: Vec<Capability>,
    pub can_receive: Vec<Capability>,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    BlindsState,
    ColorHue,
    ColorSaturation,
    ColorTemperature,
    Coordinates,
    CountryCode,
    CustomName,
    IsOn,
    LightLevel,
    LogLevel,
    PermittingJoin,
    Time,
    Timezone,
    UserConsents,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum DeviceType {
    LightController,
    Light,
    Gateway,
    MotionSensor,
    Outlet,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::LightController => f.pad("LightController"),
            Self::Light => f.pad("Light"),
            Self::Gateway => f.pad("Gateway"),
            Self::MotionSensor => f.pad("MotionSensor"),
            Self::Outlet => f.pad("Outlet"),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum Startup {
    StartOn,
    StartOff,
    StartPrevious,
    StartToggle,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub custom_name: String,
    pub firmware_version: String,
    pub hardware_version: String,
    pub manufacturer: String,
    pub model: String,
    pub ota_policy: String,
    pub ota_progress: u8,
    pub ota_schedule_end: String,
    pub ota_schedule_start: String,
    pub ota_state: String,
    pub ota_status: String,
    pub product_code: Option<String>,
    pub serial_number: String,

    // TODO: Find a better way to represent these things that are not present on all.
    // Light, controller and outlet
    pub is_on: bool,

    // Outlet and light
    pub startup_on_off: Option<Startup>,

    // Light
    pub light_level: Option<u8>,
    pub permitting_join: bool,
    pub color_mode: Option<String>,
    pub color_temperature: Option<u16>,
    pub color_temperature_min: Option<u16>,
    pub color_temperature_max: Option<u16>,
    pub startup_temperature: Option<i16>,
    pub color_hue: Option<f64>,
    pub color_saturation: Option<f64>,
    pub circadian_rhythm_mode: Option<String>,

    // Controller
    pub battery_percentage: Option<u8>,

    // Blinds and controller
    pub blinds_current_level: Option<u8>,
    pub blinds_target_level: Option<u8>,
    pub blinds_state: Option<String>,

    // EnvironmentSensor
    pub current_temperature: Option<u8>,
    pub current_r_h: Option<u8>,
    pub current_p_m25: Option<u8>,
    pub max_measured_p_m25: Option<u8>,
    pub min_measured_p_m25: Option<u8>,
    pub voc_index: Option<u8>,

    // OpenCloseSensor
    pub is_open: Option<bool>,
}

impl Device {
    pub fn into_inner(&self) -> &DeviceData {
        match self {
            Device::Blind(inner) => inner,
            Device::Controller(inner) => inner,
            Device::Gateway(inner) => inner,
            Device::Light(inner) => inner,
            Device::Outlet(inner) => inner,
            Device::Sensor(inner) => inner,
        }
    }
}

//! IKEA support multiple devices to be controlled via the Dirigera hub and they're divided into
//! several types, in this code represented as the [Device] enum.
use crate::deserialize_datetime;
use serde::{Deserialize, Serialize};

/// A [`Device`] is a resource that is able to connect to the IKEA Dirigera hub - or the actual hub
/// itself. It's represented as an enum with one variant for each type rather than separate types
/// for each content since the data for the devices are shared.
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

/// Common data that is shared between all [`Device`]s.
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
}

/// A device can have capabilities it can send or receive. Each type is represented as a list of
/// [`Capability`].
#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub can_send: Vec<Capability>,
    pub can_receive: Vec<Capability>,
}

/// Available capabilities across all devices that is listed either as something the device can
/// send or receive.
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

/// A [`Device`] has both a `type` which is interpreted as the [`Device`] enum but also a
/// `device_type`. They don't always overlap.
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

/// A device can start in different modes. It can start on, off, same as previous or toggled. This
/// is used f.ex. after a power outage.
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum Startup {
    StartOn,
    StartOff,
    StartPrevious,
    StartToggle,
}

/// The room which the [`Device`] is bound to. Icon and color represents what icon and color is
/// selected in the IKEA [iPhone](https://apps.apple.com/se/app/ikea-home-smart/id1633226273) or
/// [Android](https://play.google.com/store/apps/details?id=com.ikea.inter.homesmart.system2&hl=sv&pli=1)
/// app.
#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
}

/// Each [`Device`] has attributes that's unique to the specific [`Device`]. Here however they're
/// all represented in the same struct. Some of the attributes are common across all [`Device`] but
/// the ones that are not are defined as optional.
///
/// <div class="warning">
/// This is not optimal and will most likely change in a future version.
/// </div>
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

    // Light, controller and outlet
    pub is_on: bool,

    // Light and outlet
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

    // Environment sensor
    pub current_temperature: Option<u8>,
    pub current_r_h: Option<u8>,
    pub current_p_m25: Option<u8>,
    pub max_measured_p_m25: Option<u8>,
    pub min_measured_p_m25: Option<u8>,
    pub voc_index: Option<u8>,

    // Open and close sensor
    pub is_open: Option<bool>,
}

impl Device {
    /// Get a reference to the [`DeviceData`] for the [`Device`].
    pub fn inner(&self) -> &DeviceData {
        match self {
            Device::Blind(inner) => inner,
            Device::Controller(inner) => inner,
            Device::Gateway(inner) => inner,
            Device::Light(inner) => inner,
            Device::Outlet(inner) => inner,
            Device::Sensor(inner) => inner,
        }
    }

    /// Get a mutable reference to the [`DeviceData`] for the [`Device`].
    pub fn inner_mut(&mut self) -> &mut DeviceData {
        match self {
            Device::Blind(ref mut inner) => inner,
            Device::Controller(ref mut inner) => inner,
            Device::Gateway(ref mut inner) => inner,
            Device::Light(ref mut inner) => inner,
            Device::Outlet(ref mut inner) => inner,
            Device::Sensor(ref mut inner) => inner,
        }
    }
}

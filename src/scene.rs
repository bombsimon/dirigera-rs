//! With the IKEA Home Smart app you can configure scenes that can be either triggered manually or
//! on a schedule. Scenes are specific configuration for a set of devices such as color
//! temperature, light level, blind level etcetera.
use crate::{deserialize_datetime, deserialize_datetime_optional};
use serde::Deserialize;

/// A [`Scene`] is represented by its `type` and will hold all the [`SceneData`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Scene {
    UserScene(SceneData),
}

/// Specific data for a scene such as what actions it will do and what [`Trigger`]s it has.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneData {
    pub id: String,
    pub info: Info,
    #[serde(alias = "type")]
    pub scene_type: Option<String>,
    pub actions: Vec<Action>,
    pub commands: Vec<String>,
    pub triggers: Vec<Trigger>,
    pub undo_allowed_duration: u8,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default, deserialize_with = "deserialize_datetime_optional")]
    pub last_completed: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, deserialize_with = "deserialize_datetime_optional")]
    pub last_triggered: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, deserialize_with = "deserialize_datetime_optional")]
    pub last_undo: Option<chrono::DateTime<chrono::Utc>>,
}

/// Each scene has a name and icon which is represented under the scene [`Info`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub name: String,
    pub icon: String,
}

/// A scene can be triggered from the app (or API), based on sunrise or sunset or on a specific
/// time.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Trigger {
    App(AppTrigger),
    SunriseSunset(SunriseSunsetTrigger),
    Time(TimeTrigger),
}

/// Events triggered from the app shows the state and when it was triggered.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTrigger {
    pub id: String,
    pub disabled: bool,
    #[serde(default, deserialize_with = "deserialize_datetime_optional")]
    pub triggered_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Events triggered on time will show when the next trigger will happen and what [`EndTrigger`] the
/// schedule has.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeTrigger {
    pub id: String,
    pub disabled: bool,
    #[serde(default, deserialize_with = "deserialize_datetime")]
    pub next_trigger_at: chrono::DateTime<chrono::Utc>,
    pub trigger: Time,
    pub end_trigger_event: EndTrigger,
}

/// Sunrise and sunset events will sync with the user's location and the response will show when
/// the next trigger will happen and what [`EndTrigger`] the schedule has.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SunriseSunsetTrigger {
    pub id: String,
    pub disabled: bool,
    #[serde(default, deserialize_with = "deserialize_datetime")]
    pub next_trigger_at: chrono::DateTime<chrono::Utc>,
    pub trigger: Follow,
    pub end_trigger_event: EndTrigger,
}

/// An [`EndTrigger`] is something that will trigger the scene to end. It can be based on a
/// duration, sunrise or sunset or a specific time.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "trigger")]
pub enum EndTrigger {
    Duration(Duration),
    SunriseSunset(Follow),
    Time(Time),
}

/// Duration is just number of seconds from the trigger start.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Duration {
    pub duration: u32,
}

/// Sunrise and sunset shows what days to trigger for sunrise or sunset if specific days and any
/// offset from the sunrise or sunset time.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Follow {
    Sunrise {
        days: Option<Vec<String>>,
        offset: i32,
    },
    Sunset {
        days: Option<Vec<String>>,
        offset: i32,
    },
}

/// Time shows what days to trigger for the specific time and what time that is.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Time {
    pub days: Option<Vec<String>>,
    pub time: String,
}

/// A scene has a type to target for its action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Action {
    Device(ActionData),
}

/// Data for the action type which holds the [`Device`](crate::Device) id and attribute for the [`Scene`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionData {
    pub id: String,
    pub device_id: String,
    pub attributes: SceneAttributes,
}

/// Attributes to the scene which shows information about on or off state and light level and color
/// temperature for [`Device`](crate::Device)s that support those.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneAttributes {
    pub is_on: bool,
    pub light_level: Option<u8>,
    pub color_temperature: Option<u16>,
}

impl Scene {
    /// Get a reference to the [`SceneData`] for the [`Scene`].
    pub fn inner(&self) -> &SceneData {
        match self {
            Scene::UserScene(inner) => inner,
        }
    }
}

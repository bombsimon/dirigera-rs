use crate::{deserialize_datetime, deserialize_datetime_optional};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Scene {
    UserScene(SceneData),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneData {
    pub id: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Trigger {
    App(AppTrigger),
    SunriseSunset(SunriseSunsetTrigger),
    Time(TimeTrigger),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTrigger {
    pub id: String,
    pub disabled: bool,
    #[serde(default, deserialize_with = "deserialize_datetime_optional")]
    pub triggered_at: Option<chrono::DateTime<chrono::Utc>>,
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "trigger")]
pub enum EndTrigger {
    Duration(Duration),
    SunriseSunset(Follow),
    Time(Time),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Duration {
    pub duration: u32,
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Time {
    pub days: Option<Vec<String>>,
    pub time: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Action {
    Device(ActionData),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionData {
    pub id: String,
    pub device_id: String,
    pub attributes: SceneAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneAttributes {
    pub is_on: bool,
    pub light_level: Option<u8>,
    pub color_temperature: Option<u16>,
}

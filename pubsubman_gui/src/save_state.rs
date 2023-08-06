use std::collections::HashMap;

use pubsubman_backend::model::{PubsubMessage, TopicName};

use crate::{column_settings::ColumnSettings, settings::Settings};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SaveState {
    pub messages: HashMap<TopicName, Vec<PubsubMessage>>,
    pub column_settings: HashMap<TopicName, ColumnSettings>,
    pub settings: Settings,
}

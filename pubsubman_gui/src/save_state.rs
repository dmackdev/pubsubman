use std::collections::HashMap;

use pubsubman_backend::model::{PubsubMessage, TopicName};

use crate::settings::Settings;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SaveState {
    pub messages: HashMap<TopicName, Vec<PubsubMessage>>,
    pub settings: Settings,
}

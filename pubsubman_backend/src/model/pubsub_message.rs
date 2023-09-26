use chrono::{DateTime, TimeZone, Utc};
use google_cloud_pubsub::subscriber::ReceivedMessage;
use serde_json::Value;
use std::{collections::HashMap, str};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PubsubMessage {
    pub id: String,
    pub publish_time: Option<DateTime<Utc>>,
    pub data: String,
    pub data_json: Value,
    pub attributes: HashMap<String, String>,
}

impl From<ReceivedMessage> for PubsubMessage {
    fn from(value: ReceivedMessage) -> Self {
        let publish_time = value
            .message
            .publish_time
            .map(|t| Utc.timestamp_opt(t.seconds, t.nanos.try_into().unwrap_or(0)))
            .and_then(|lr| match lr {
                chrono::LocalResult::Single(dt) => Some(dt),
                _ => None,
            });

        let data = str::from_utf8(&value.message.data).unwrap().to_string();

        let data_json: Value = match serde_json::from_str(&data) {
            Ok(val) => val,
            Err(_) => Value::String(data.clone()),
        };

        Self {
            id: value.message.message_id,
            publish_time,
            data,
            data_json,
            attributes: value.message.attributes,
        }
    }
}

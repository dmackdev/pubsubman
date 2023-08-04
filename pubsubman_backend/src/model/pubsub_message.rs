use chrono::{DateTime, TimeZone, Utc};
use google_cloud_pubsub::subscriber::ReceivedMessage;
use std::str;

#[derive(Debug)]
pub struct PubsubMessage {
    pub id: String,
    pub publish_time: Option<DateTime<Utc>>,
    pub data: String,
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

        Self {
            id: value.message.message_id,
            publish_time,
            data,
        }
    }
}

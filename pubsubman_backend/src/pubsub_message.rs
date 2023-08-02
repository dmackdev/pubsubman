use google_cloud_pubsub::subscriber::ReceivedMessage;
use std::str;
use time::{Duration, OffsetDateTime, Time};

#[derive(Debug)]
pub struct PubsubMessage {
    pub publish_time: Option<Time>,
    pub data: String,
}

impl From<ReceivedMessage> for PubsubMessage {
    fn from(value: ReceivedMessage) -> Self {
        let publish_time = value
            .message
            .publish_time
            .map(|t| create_time_from_seconds_and_nanoseconds(t.seconds, t.nanos));

        let data = str::from_utf8(&value.message.data).unwrap().to_string();

        Self { publish_time, data }
    }
}

fn create_time_from_seconds_and_nanoseconds(seconds: i64, nanoseconds: i32) -> Time {
    let duration = Duration::seconds(seconds) + Duration::nanoseconds(nanoseconds as i64);
    OffsetDateTime::from_unix_timestamp(duration.whole_seconds())
        .unwrap()
        .time()
}

#[derive(Debug)]
pub struct PubsubMessageToPublish {
    data: String,
}

impl PubsubMessageToPublish {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}

impl From<PubsubMessageToPublish> for google_cloud_googleapis::pubsub::v1::PubsubMessage {
    fn from(val: PubsubMessageToPublish) -> Self {
        Self {
            data: val.data.into(),
            ..Default::default()
        }
    }
}

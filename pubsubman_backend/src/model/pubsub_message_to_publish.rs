use std::collections::HashMap;

#[derive(Debug)]
pub struct PubsubMessageToPublish {
    data: String,
    attributes: HashMap<String, String>,
}

impl PubsubMessageToPublish {
    pub fn new(data: String, attributes: HashMap<String, String>) -> Self {
        Self { data, attributes }
    }
}

impl From<PubsubMessageToPublish> for google_cloud_googleapis::pubsub::v1::PubsubMessage {
    fn from(val: PubsubMessageToPublish) -> Self {
        Self {
            data: val.data.into(),
            attributes: val.attributes,
            ..Default::default()
        }
    }
}

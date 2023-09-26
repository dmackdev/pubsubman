mod pubsub_message;
mod pubsub_message_to_publish;

use std::fmt::Display;

pub use pubsub_message::PubsubMessage;
pub use pubsub_message_to_publish::PubsubMessageToPublish;

#[derive(
    Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct TopicName(pub String);

impl Display for TopicName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct SubscriptionName(pub String);

impl Display for SubscriptionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

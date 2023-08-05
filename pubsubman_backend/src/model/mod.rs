mod pubsub_message;
mod pubsub_message_to_publish;

pub use pubsub_message::PubsubMessage;
pub use pubsub_message_to_publish::PubsubMessageToPublish;

#[derive(
    Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct TopicName(pub String);

#[derive(
    Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct SubscriptionName(pub String);

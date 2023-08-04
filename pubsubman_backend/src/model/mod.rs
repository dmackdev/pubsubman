mod pubsub_message;
pub use pubsub_message::PubsubMessage;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TopicName(pub String);

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SubscriptionName(pub String);

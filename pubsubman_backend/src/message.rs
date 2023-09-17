use tokio_util::sync::CancellationToken;

use crate::model::{PubsubMessage, PubsubMessageToPublish, SubscriptionName, TopicName};

#[derive(Debug)]
pub enum FrontendMessage {
    RefreshTopicsRequest,
    CreateSubscriptionRequest(TopicName),
    DeleteSubscriptions(Vec<SubscriptionName>),
    StreamMessages(TopicName, SubscriptionName, CancellationToken),
    PublishMessage(TopicName, PubsubMessageToPublish),
}

#[derive(Debug)]
pub enum BackendMessage {
    ClientInitialised(String),
    TopicsUpdated(Vec<TopicName>),
    SubscriptionCreated(TopicName, SubscriptionName),
    MessageReceived(TopicName, PubsubMessage),
    SubscriptionsDeleted(Vec<Result<SubscriptionName, SubscriptionName>>),
    Error(BackendError),
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Failed to initialise client.")]
    ClientInitFailed,
    #[error("Failed to get topics.")]
    GetTopicsFailed,
    #[error("Failed to create Subscription for {0}.")]
    CreateSubscriptionFailed(TopicName),
    #[error("Failed to get messages from {0}.")]
    StreamMessagesFailed(TopicName, SubscriptionName),
    #[error("Failed to publish message to {0}.")]
    PublishMessageFailed(TopicName),
}

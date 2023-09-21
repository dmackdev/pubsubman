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
    TopicsUpdated(Vec<TopicName>),
    SubscriptionCreated(TopicName, SubscriptionName),
    MessageReceived(TopicName, PubsubMessage),
    SubscriptionsDeleted(Vec<Result<SubscriptionName, SubscriptionName>>),
    Error(BackendError),
}

#[derive(Debug)]
pub enum BackendError {
    GetTopicsFailed,
    CreateSubscriptionFailed(TopicName),
    StreamMessagesFailed(TopicName, SubscriptionName),
    PublishMessageFailed(TopicName),
}

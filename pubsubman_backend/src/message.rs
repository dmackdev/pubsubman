use tokio_util::sync::CancellationToken;

use crate::model::{PubsubMessage, PubsubMessageToPublish, SubscriptionName, TopicName};

#[derive(Debug)]
pub enum FrontendMessage {
    RefreshTopicsRequest,
    CreateSubscriptionRequest(TopicName),
    StreamMessages(TopicName, SubscriptionName, CancellationToken),
    PublishMessage(TopicName, PubsubMessageToPublish),
}

#[derive(Debug)]
pub enum BackendMessage {
    TopicsUpdated(Vec<TopicName>),
    SubscriptionCreated(TopicName, SubscriptionName),
    MessageReceived(TopicName, PubsubMessage),
}

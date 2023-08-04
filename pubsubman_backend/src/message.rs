use tokio_util::sync::CancellationToken;

use crate::model::{PubsubMessage, SubscriptionName, TopicName};

#[derive(Debug)]
pub enum FrontendMessage {
    RefreshTopicsRequest,
    CreateSubscriptionRequest(TopicName),
    PullMessages(TopicName, SubscriptionName, CancellationToken),
}

#[derive(Debug)]
pub enum BackendMessage {
    TopicsUpdated(Vec<TopicName>),
    SubscriptionCreated(TopicName, SubscriptionName),
    MessageReceived(TopicName, PubsubMessage),
}

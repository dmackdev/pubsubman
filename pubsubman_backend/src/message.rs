use tokio_util::sync::CancellationToken;

use crate::model::PubsubMessage;

#[derive(Debug)]
pub enum FrontendMessage {
    RefreshTopicsRequest,
    CreateSubscriptionRequest(String),
    Subscribe(String, CancellationToken),
}

#[derive(Debug)]
pub enum BackendMessage {
    TopicsUpdated(Vec<String>),
    SubscriptionCreated(String, String),
    MessageReceived(String, PubsubMessage),
}

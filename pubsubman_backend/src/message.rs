use crate::pubsub_message::PubsubMessage;

#[derive(Debug)]
pub enum FrontendMessage {
    RefreshTopicsRequest,
    CreateSubscriptionRequest(String),
    PullMessages(String),
}

#[derive(Debug)]
pub enum BackendMessage {
    TopicsUpdated(Vec<String>),
    SubscriptionCreated(String, String),
    MessageReceived(String, PubsubMessage),
}

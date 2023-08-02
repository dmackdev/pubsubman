#[derive(Debug)]
pub enum FrontendMessage {
    RefreshTopicsRequest,
    CreateSubscriptionRequest(String),
}

#[derive(Debug)]
pub enum BackendMessage {
    TopicsUpdated(Vec<String>),
    SubscriptionCreated(String, String),
}

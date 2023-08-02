#[derive(Debug)]
pub enum FrontendMessage {
    RefreshTopicsRequest,
}

#[derive(Debug)]
pub enum BackendMessage {
    TopicsUpdated(Vec<String>),
}

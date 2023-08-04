use pubsubman_backend::model::PubsubMessage;

pub struct Subscription {
    pub id: String,
    pub messages: Vec<PubsubMessage>,
}

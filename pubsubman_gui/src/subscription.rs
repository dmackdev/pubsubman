use pubsubman_backend::pubsub_message::PubsubMessage;

pub struct Subscription {
    pub id: String,
    pub messages: Vec<PubsubMessage>,
}

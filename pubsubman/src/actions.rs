use std::time::Duration;

use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessageToPublish, SubscriptionName, TopicName},
};
use tokio::{sync::mpsc::Sender, time::sleep};
use tokio_util::sync::CancellationToken;

pub fn refresh_topics(front_tx: &Sender<FrontendMessage>, delay_millis: Option<u64>) {
    let front_tx = front_tx.to_owned();

    tokio::spawn(async move {
        if let Some(delay_millis) = delay_millis {
            sleep(Duration::from_millis(delay_millis)).await;
        }
        let _ = front_tx.send(FrontendMessage::RefreshTopicsRequest).await;
    });
}

pub fn create_subscription(front_tx: &Sender<FrontendMessage>, topic_name: &TopicName) {
    let front_tx = front_tx.to_owned();
    let topic_name = topic_name.to_owned();

    tokio::spawn(async move {
        let _ = front_tx
            .send(FrontendMessage::CreateSubscriptionRequest(topic_name))
            .await;
    });
}

pub fn delete_subscriptions(front_tx: &Sender<FrontendMessage>, sub_names: Vec<SubscriptionName>) {
    let front_tx = front_tx.to_owned();

    tokio::spawn(async move {
        let _ = front_tx
            .send(FrontendMessage::DeleteSubscriptions(sub_names))
            .await;
    });
}

pub fn pull_message_batch(
    front_tx: &Sender<FrontendMessage>,
    topic_name: &TopicName,
    sub_name: &SubscriptionName,
    cancel_token: &CancellationToken,
) {
    stream_messages(front_tx, topic_name, sub_name, cancel_token);

    let cancel_token_clone = cancel_token.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        cancel_token_clone.cancel();
    });
}

pub fn stream_messages(
    front_tx: &Sender<FrontendMessage>,
    topic_name: &TopicName,
    sub_name: &SubscriptionName,
    cancel_token: &CancellationToken,
) {
    let topic_name = topic_name.to_owned();
    let sub_name = sub_name.to_owned();
    let front_tx = front_tx.to_owned();
    let cancel_token = cancel_token.to_owned();

    tokio::spawn(async move {
        front_tx
            .send(FrontendMessage::StreamMessages(
                topic_name,
                sub_name,
                cancel_token,
            ))
            .await
            .unwrap();
    });
}

pub fn publish_message(
    front_tx: &Sender<FrontendMessage>,
    topic_name: &TopicName,
    message: PubsubMessageToPublish,
) {
    let front_tx = front_tx.to_owned();
    let topic_name = topic_name.to_owned();

    tokio::spawn(async move {
        front_tx
            .send(FrontendMessage::PublishMessage(topic_name, message))
            .await
            .unwrap();
    });
}

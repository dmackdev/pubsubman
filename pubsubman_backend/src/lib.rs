use futures_util::StreamExt;
use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    subscription::SubscriptionConfig,
};
use message::{BackendMessage, FrontendMessage};
use model::{SubscriptionName, TopicName};
use tokio::{
    runtime::Builder,
    select,
    sync::mpsc::{Receiver, Sender},
};
use uuid::Uuid;

pub mod message;
pub mod model;

pub struct Backend {
    back_tx: Sender<BackendMessage>,
    front_rx: Receiver<FrontendMessage>,
}

impl Backend {
    pub fn new(back_tx: Sender<BackendMessage>, front_rx: Receiver<FrontendMessage>) -> Self {
        Self { back_tx, front_rx }
    }

    pub fn init(&mut self) {
        let rt = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        loop {
            if let Ok(message) = self.front_rx.try_recv() {
                match message {
                    FrontendMessage::RefreshTopicsRequest => {
                        let back_tx = self.back_tx.clone();

                        rt.spawn(async move {
                            let client = create_client().await;
                            let topics = client
                                .get_topics(None)
                                .await
                                .unwrap()
                                .into_iter()
                                .map(TopicName)
                                .collect();

                            back_tx
                                .send(BackendMessage::TopicsUpdated(topics))
                                .await
                                .unwrap();
                        });
                    }
                    FrontendMessage::CreateSubscriptionRequest(topic_name) => {
                        let back_tx = self.back_tx.clone();

                        rt.spawn(async move {
                            let client = create_client().await;

                            let sub_name = format!("pubsubman-subscription-{}", Uuid::new_v4());

                            let _subscription = client
                                .create_subscription(
                                    &sub_name,
                                    &topic_name.0,
                                    SubscriptionConfig::default(),
                                    None,
                                )
                                .await
                                .unwrap();

                            back_tx
                                .send(BackendMessage::SubscriptionCreated(
                                    topic_name,
                                    SubscriptionName(sub_name),
                                ))
                                .await
                                .unwrap();
                        });
                    }
                    FrontendMessage::StreamMessages(topic_name, sub_name, cancel_token) => {
                        let back_tx = self.back_tx.clone();

                        rt.spawn(async move {
                            let client = create_client().await;
                            let subscription = client.subscription(&sub_name.0);

                            let pull_messages_future = async move {
                                let mut stream = subscription.subscribe(None).await.unwrap();

                                while let Some(message) = stream.next().await {
                                    let _ = message.ack().await;

                                    back_tx
                                        .send(BackendMessage::MessageReceived(
                                            topic_name.clone(),
                                            message.into(),
                                        ))
                                        .await
                                        .unwrap();
                                }
                            };

                            select! {
                              _ = cancel_token.cancelled() => {}
                              _ = pull_messages_future => {}
                            }
                        });
                    }
                    FrontendMessage::PublishMessage(topic_name, message) => {
                        rt.spawn(async move {
                            let client = create_client().await;
                            let topic = client.topic(&topic_name.0);
                            let publisher = topic.new_publisher(None);
                            let awaiter = publisher.publish(message.into()).await;
                            awaiter.get().await
                        });
                    }
                }
            }
        }
    }
}

async fn create_client() -> Client {
    let config = ClientConfig::default().with_auth().await.unwrap();
    Client::new(config).await.unwrap()
}

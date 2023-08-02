use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    subscription::SubscriptionConfig,
};
use message::{BackendMessage, FrontendMessage};
use tokio::{
    runtime::Builder,
    sync::mpsc::{Receiver, Sender},
};
use uuid::Uuid;

pub mod message;

pub struct Backend {
    back_tx: Sender<BackendMessage>,
    front_rx: Receiver<FrontendMessage>,
}

impl Backend {
    pub async fn new(back_tx: Sender<BackendMessage>, front_rx: Receiver<FrontendMessage>) -> Self {
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
                            let topics = client.get_topics(None).await.unwrap();
                            back_tx
                                .send(BackendMessage::TopicsUpdated(topics))
                                .await
                                .unwrap();
                        });
                    }
                    FrontendMessage::CreateSubscriptionRequest(topic_id) => {
                        let back_tx = self.back_tx.clone();

                        rt.spawn(async move {
                            let client = create_client().await;

                            let sub_id = format!("pubsubman-subscription-{}", Uuid::new_v4());

                            let _subscription = client
                                .create_subscription(
                                    &sub_id,
                                    &topic_id,
                                    SubscriptionConfig::default(),
                                    None,
                                )
                                .await
                                .unwrap();

                            back_tx
                                .send(BackendMessage::SubscriptionCreated(topic_id, sub_id))
                                .await
                                .unwrap();
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

use google_cloud_pubsub::client::{Client, ClientConfig};
use message::{BackendMessage, FrontendMessage};
use tokio::{
    runtime::Builder,
    sync::mpsc::{Receiver, Sender},
};

pub mod message;

pub struct Backend {
    back_tx: Sender<BackendMessage>,
    front_rx: Receiver<FrontendMessage>,
    client: Client,
}

impl Backend {
    pub async fn new(back_tx: Sender<BackendMessage>, front_rx: Receiver<FrontendMessage>) -> Self {
        let config = ClientConfig::default().with_auth().await.unwrap();
        let client = Client::new(config).await.unwrap();

        Self {
            back_tx,
            front_rx,
            client,
        }
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
                        let client = self.client.clone();
                        let back_tx = self.back_tx.clone();

                        rt.spawn(async move {
                            let topics = client.get_topics(None).await.unwrap();
                            back_tx
                                .send(BackendMessage::TopicsUpdated(topics))
                                .await
                                .unwrap();
                        });
                    }
                }
            }
        }
    }
}

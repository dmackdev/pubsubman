use std::{error::Error, sync::Arc};

use futures_util::StreamExt;
use google_cloud_gax::conn::Environment;
use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    subscription::SubscriptionConfig,
};
use message::{BackendError, BackendMessage, FrontendMessage};
use model::{PubsubMessageToPublish, SubscriptionName, TopicName};
use tokio::{
    runtime::{Builder, Runtime},
    select,
    sync::mpsc::{Receiver, Sender},
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub mod message;
pub mod model;

pub struct Backend {
    back_tx: Sender<BackendMessage>,
    front_rx: Receiver<FrontendMessage>,
    client: Arc<Client>,
    // Store and reuse the same runtime (that created the client) for async operations,
    // because the gPRC service appears to require the same runtime that created it:
    // https://github.com/hyperium/tonic/issues/942#issuecomment-1313396286
    rt: Runtime,
}

impl Backend {
    pub fn new(
        back_tx: Sender<BackendMessage>,
        front_rx: Receiver<FrontendMessage>,
        emulator_project_id: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        let rt = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        match rt.block_on(async { create_client(emulator_project_id).await }) {
            Ok((client, project_id)) => {
                let back_tx_clone = back_tx.clone();
                rt.spawn(async move {
                    back_tx_clone
                        .send(BackendMessage::ClientInitialised(project_id))
                        .await
                        .unwrap();
                });
                Ok(Self {
                    back_tx,
                    front_rx,
                    client: Arc::new(client),
                    rt,
                })
            }
            Err(err) => {
                rt.spawn(async move {
                    back_tx
                        .send(BackendMessage::Error(BackendError::ClientInitFailed))
                        .await
                        .unwrap();
                });
                Err(err)
            }
        }
    }

    pub fn init(&mut self) {
        while let Some(message) = self.front_rx.blocking_recv() {
            match message {
                FrontendMessage::RefreshTopicsRequest => {
                    self.get_topics();
                }
                FrontendMessage::CreateSubscriptionRequest(topic_name) => {
                    self.create_subscription(topic_name);
                }
                FrontendMessage::DeleteSubscriptions(sub_names) => {
                    self.delete_subscriptions(sub_names);
                }
                FrontendMessage::StreamMessages(topic_name, sub_name, cancel_token) => {
                    self.stream_messages(topic_name, sub_name, cancel_token);
                }
                FrontendMessage::PublishMessage(topic_name, message) => {
                    self.publish_message(topic_name, message);
                }
            }
        }
    }

    fn get_topics(&self) {
        let back_tx = self.back_tx.clone();
        let client = self.client.clone();

        self.rt.spawn(async move {
            let message = client
                .get_topics(None)
                .await
                .map(|topics| {
                    BackendMessage::TopicsUpdated(topics.into_iter().map(TopicName).collect())
                })
                .unwrap_or_else(|status| {
                    eprintln!("{}", status);
                    BackendMessage::Error(BackendError::GetTopicsFailed)
                });

            back_tx.send(message).await.unwrap();
        });
    }

    fn create_subscription(&self, topic_name: TopicName) {
        let back_tx = self.back_tx.clone();
        let client = self.client.clone();

        self.rt.spawn(async move {
            let message = match client
                .create_subscription(
                    &format!("pubsubman-subscription-{}", Uuid::new_v4()),
                    &topic_name.0,
                    SubscriptionConfig::default(),
                    None,
                )
                .await
            {
                Ok(subscription) => {
                    let fq_sub_name = subscription.fully_qualified_name().to_owned();
                    BackendMessage::SubscriptionCreated(topic_name, SubscriptionName(fq_sub_name))
                }
                Err(status) => {
                    eprintln!("{}", status);
                    BackendMessage::Error(BackendError::CreateSubscriptionFailed(topic_name))
                }
            };

            back_tx.send(message).await.unwrap();
        });
    }

    fn delete_subscriptions(&self, sub_names: Vec<SubscriptionName>) {
        let back_tx = self.back_tx.clone();
        let client = self.client.clone();

        self.rt.spawn(async move {
            let futures = sub_names.into_iter().map(|sub_name| {
                let client = client.clone();
                async move {
                    let subscription = client.subscription(&sub_name.0);
                    match subscription.delete(None).await {
                        Ok(_) => Ok(sub_name),
                        Err(status) => {
                            eprintln!("{}", status);
                            Err(sub_name)
                        }
                    }
                }
            });

            let results = futures::future::join_all(futures).await;

            back_tx
                .send(BackendMessage::SubscriptionsDeleted(results))
                .await
                .unwrap();
        });
    }

    fn stream_messages(
        &self,
        topic_name: TopicName,
        sub_name: SubscriptionName,
        cancel_token: CancellationToken,
    ) {
        let back_tx = self.back_tx.clone();
        let client = self.client.clone();

        self.rt.spawn(async move {
            let subscription = client.subscription(&sub_name.0);

            let pull_messages_future = async move {
                match subscription.subscribe(None).await {
                    Ok(mut stream) => {
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
                    }
                    Err(status) => {
                        eprintln!("{}", status);
                        back_tx
                            .send(BackendMessage::Error(BackendError::StreamMessagesFailed(
                                topic_name, sub_name,
                            )))
                            .await
                            .unwrap()
                    }
                };
            };

            select! {
              _ = cancel_token.cancelled() => {}
              _ = pull_messages_future => {}
            }
        });
    }

    fn publish_message(&self, topic_name: TopicName, message: PubsubMessageToPublish) {
        let back_tx = self.back_tx.clone();
        let client = self.client.clone();

        self.rt.spawn(async move {
            let topic = client.topic(&topic_name.0);
            let publisher = topic.new_publisher(None);
            let awaiter = publisher.publish(message.into()).await;

            if let Err(status) = awaiter.get().await {
                eprintln!("{}", status);
                back_tx
                    .send(BackendMessage::Error(BackendError::PublishMessageFailed(
                        topic_name,
                    )))
                    .await
                    .unwrap()
            }
        });
    }
}

async fn create_client(
    emulator_project_id: Option<String>,
) -> Result<(Client, String), Box<dyn Error>> {
    let mut config = ClientConfig::default().with_auth().await?;

    if let (Environment::Emulator(_), Some(emulator_project_id)) =
        (&config.environment, emulator_project_id)
    {
        config.project_id = Some(emulator_project_id);
    }

    let project_id = config.project_id.clone();

    Ok((Client::new(config).await?, project_id.unwrap()))
}

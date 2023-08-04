use std::collections::HashMap;

use chrono::{DateTime, Local};
use pubsubman_backend::{
    message::{BackendMessage, FrontendMessage},
    model::{PubsubMessage, SubscriptionName, TopicName},
    Backend,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use crate::ui::render_topic_name;

struct TopicViewState {
    selected_topic_id: TopicName,
    stream_messages_enabled: bool,
    stream_messages_cancel_token: Option<CancellationToken>,
}

impl TopicViewState {
    fn new(selected_topic_id: TopicName) -> Self {
        Self {
            selected_topic_id,
            stream_messages_enabled: false,
            stream_messages_cancel_token: None,
        }
    }
}

pub struct TemplateApp {
    topics: Vec<TopicName>,
    topic_view: Option<TopicViewState>,
    /// The subscriptions this app has created in order to recieve messages.
    /// Mapping of topic ID -> Subscription.
    subscriptions: HashMap<TopicName, SubscriptionName>,
    messages: HashMap<TopicName, Vec<PubsubMessage>>,
    front_tx: Sender<FrontendMessage>,
    back_rx: Receiver<BackendMessage>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (front_tx, front_rx) = tokio::sync::mpsc::channel(10);
        let (back_tx, back_rx) = tokio::sync::mpsc::channel(10);

        tokio::spawn(async move {
            Backend::new(back_tx, front_rx).await.init();
        });

        let front_tx_clone = front_tx.clone();
        tokio::spawn(async move {
            let _ = front_tx_clone
                .send(FrontendMessage::RefreshTopicsRequest)
                .await;
        });

        Self {
            topics: vec![],
            topic_view: None,
            subscriptions: HashMap::new(),
            messages: HashMap::new(),
            front_tx,
            back_rx,
        }
    }

    fn handle_backend_message(&mut self) {
        match self.back_rx.try_recv() {
            Ok(message) => match message {
                BackendMessage::TopicsUpdated(topics) => {
                    self.topics = topics;

                    let front_tx = self.front_tx.clone();
                    tokio::spawn(async move {
                        let _ = front_tx.send(FrontendMessage::RefreshTopicsRequest).await;
                    });
                }
                BackendMessage::SubscriptionCreated(topic_id, sub_id) => {
                    self.subscriptions.insert(topic_id, sub_id);
                }
                BackendMessage::MessageReceived(topic_id, message) => {
                    self.messages.entry(topic_id).or_default().push(message);
                }
            },
            Err(_err) => {} //println!("{:?}", err),
        }
    }

    fn render_top_panel(&self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });
    }

    fn render_topics_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .exact_width(250.0)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Topics");

                    let topics = self.topics.clone();
                    for topic in topics {
                        let is_selected = self.is_topic_selected(&topic);

                        render_topic_name(ui, &topic, is_selected, || self.on_topic_click(&topic));
                    }
                });
            });
    }

    fn on_topic_click(&mut self, topic_id: &TopicName) {
        if self.is_topic_selected(topic_id) {
            return;
        }

        if let Some(cancel_token) = self
            .topic_view
            .take()
            .and_then(|topic_view| topic_view.stream_messages_cancel_token)
        {
            cancel_token.cancel();
        }

        self.topic_view = Some(TopicViewState::new(topic_id.clone()));

        if !self.subscriptions.contains_key(topic_id) {
            let front_tx = self.front_tx.clone();
            let topic_id = topic_id.clone();

            tokio::spawn(async move {
                let _ = front_tx
                    .send(FrontendMessage::CreateSubscriptionRequest(topic_id))
                    .await;
            });
        }
    }

    fn is_topic_selected(&self, topic_id: &TopicName) -> bool {
        self.topic_view
            .as_ref()
            .is_some_and(|topic_view| &topic_view.selected_topic_id == topic_id)
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.topic_view.as_mut() {
                Some(topic_view) => {
                    egui::TopBottomPanel::top("topic_view_top_panel").show_inside(ui, |ui| {
                        ui.heading(&topic_view.selected_topic_id.0);
                    });

                    match self.subscriptions.get(&topic_view.selected_topic_id) {
                        Some(subscription) => {
                            egui::TopBottomPanel::bottom("topic_view_bottom_panel")
                                .exact_height(250.0)
                                .show_inside(ui, |ui| {
                                    ui.heading("Publish");
                                });

                            egui::CentralPanel::default().show_inside(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.heading("Messages");

                                    ui.add_enabled_ui(!topic_view.stream_messages_enabled, |ui| {

                                      let pull_button = ui.button("Pull");
                                      if pull_button.clicked() {
                                        let front_tx = self.front_tx.clone();
                                        let topic_id = topic_view.selected_topic_id.clone();
                                        let sub_id = subscription.clone();
                                        let cancel_token = CancellationToken::new();
                                        let cancel_token_clone = cancel_token.clone();

                                        tokio::spawn(async move {
                                          front_tx
                                          .send(FrontendMessage::Subscribe(topic_id, sub_id, cancel_token))
                                          .await
                                          .unwrap();
                                        });

                                        tokio::spawn(async move {
                                          tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                          cancel_token_clone.cancel();
                                        });
                                      }
                                      pull_button.on_hover_text(
                                          "Retrieve batch of all undelivered messages on this subscription.",
                                        ).on_disabled_hover_text("Disable Stream mode to Pull messages.");
                                      });

                                      let stream_mode_toggle = ui.toggle_value(
                                        &mut topic_view.stream_messages_enabled,
                                        "Stream",
                                      );

                                      if stream_mode_toggle.changed() {
                                        if topic_view.stream_messages_enabled {
                                          let front_tx = self.front_tx.clone();
                                          let topic_id = topic_view.selected_topic_id.clone();
                                          let sub_id = subscription.clone();
                                          let cancel_token = CancellationToken::new();
                                          let cancel_token_clone = cancel_token.clone();

                                          tokio::spawn(async move {
                                            front_tx
                                            .send(FrontendMessage::Subscribe(topic_id, sub_id, cancel_token))
                                            .await
                                            .unwrap();
                                          });

                                          topic_view.stream_messages_cancel_token = Some(cancel_token_clone);
                                        } else if let Some(cancel_token) = topic_view.stream_messages_cancel_token.take() {
                                          cancel_token.cancel();
                                        }
                                      }

                                      stream_mode_toggle.on_hover_text("Continuously retrieve messages delivered to this subscription.");
                                });

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for message in self.messages.get(&topic_view.selected_topic_id).unwrap_or(&vec![]).iter() {
                                            egui::Frame::none()
                                                .stroke(egui::Stroke::new(
                                                    0.0,
                                                    egui::Color32::DARK_BLUE,
                                                ))
                                                .show(ui, |ui| {
                                                    if let Some(publish_time) = message.publish_time
                                                    {
                                                        let local_publish_time: DateTime<Local> =
                                                            publish_time.into();

                                                        ui.label(format!(
                                                            "{}",
                                                            local_publish_time
                                                                .format("%d/%m/%Y %H:%M")
                                                        ));
                                                    }
                                                    ui.label(&message.data);
                                                });
                                        }
                                    });
                            });
                        }
                        None => {
                            ui.vertical_centered(|ui| {
                                ui.allocate_space(ui.available_size() / 2.0);
                                ui.spinner();
                            });
                        }
                    }
                }
                None => {
                    ui.vertical_centered(|ui| {
                        ui.allocate_space(ui.available_size() / 2.0);
                        ui.heading("Select a Topic.");
                    });
                }
            };
        });
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.handle_backend_message();
        self.render_top_panel(ctx, frame);
        self.render_topics_panel(ctx);
        self.render_central_panel(ctx);
    }
}

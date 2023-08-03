use std::collections::HashMap;

use chrono::{DateTime, Local};
use pubsubman_backend::{
    message::{BackendMessage, FrontendMessage},
    Backend,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use crate::{subscription::Subscription, topic::Topic};

struct TopicViewState {
    selected_topic_id: String,
    stream_messages_enabled: bool,
}

impl TopicViewState {
    fn new(selected_topic_id: String) -> Self {
        Self {
            selected_topic_id,
            stream_messages_enabled: false,
        }
    }
}

pub struct TemplateApp {
    topics: Vec<Topic>,
    topic_view: Option<TopicViewState>,
    /// The subscriptions this app has created in order to recieve messages.
    /// Mapping of topic ID -> Subscription.
    subscriptions: HashMap<String, Subscription>,
    front_tx: Sender<FrontendMessage>,
    back_rx: Receiver<BackendMessage>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (front_tx, front_rx) = tokio::sync::mpsc::channel(2);
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
            front_tx,
            back_rx,
        }
    }

    fn handle_backend_message(&mut self) {
        match self.back_rx.try_recv() {
            Ok(message) => match message {
                BackendMessage::TopicsUpdated(topics) => {
                    self.topics = topics.into_iter().map(|id| Topic { id }).collect();

                    let front_tx = self.front_tx.clone();
                    tokio::spawn(async move {
                        let _ = front_tx.send(FrontendMessage::RefreshTopicsRequest).await;
                    });
                }
                BackendMessage::SubscriptionCreated(topic_id, sub_id) => {
                    self.subscriptions.insert(
                        topic_id,
                        Subscription {
                            id: sub_id,
                            messages: vec![],
                        },
                    );
                }
                BackendMessage::MessageReceived(sub_id, message) => {
                    if let Some(subscription) =
                        self.subscriptions.values_mut().find(|s| s.id == sub_id)
                    {
                        subscription.messages.push(message);
                    }
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

                    for topic in self.topics.iter() {
                        let is_selected = self
                            .topic_view
                            .as_ref()
                            .is_some_and(|topic_view| *topic_view.selected_topic_id == topic.id);

                        let on_click = || {
                            self.topic_view = Some(TopicViewState::new(topic.id.to_string()));

                            if !self.subscriptions.contains_key(&topic.id) {
                                let front_tx = self.front_tx.clone();
                                let topic_id = topic.id.to_string();

                                tokio::spawn(async move {
                                    let _ = front_tx
                                        .send(FrontendMessage::CreateSubscriptionRequest(topic_id))
                                        .await;
                                });
                            }
                        };

                        topic.show(ui, is_selected, on_click);
                    }
                });
            });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.topic_view.as_mut() {
                Some(topic_view) => {
                    egui::TopBottomPanel::top("topic_view_top_panel").show_inside(ui, |ui| {
                        ui.heading(&topic_view.selected_topic_id);
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
                                        let sub_id = subscription.id.clone();
                                        let cancel_token = CancellationToken::new();
                                        let cancel_token_clone = cancel_token.clone();

                                        tokio::spawn(async move {
                                          front_tx
                                          .send(FrontendMessage::Subscribe(sub_id, cancel_token))
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

                                      let stream_mode_toggle = ui.toggle_value( &mut topic_view.stream_messages_enabled,
                                        "Stream",
                                      );

                                      if stream_mode_toggle.changed() && topic_view.stream_messages_enabled {
                                        let front_tx = self.front_tx.clone();
                                        let sub_id = subscription.id.clone();
                                        let cancel_token = CancellationToken::new();

                                        tokio::spawn(async move {
                                          front_tx
                                          .send(FrontendMessage::Subscribe(sub_id, cancel_token))
                                          .await
                                          .unwrap();
                                        });

                                        // TODO: Store the cancellation token and cancel it when steam mode is disabled or this topic view is closed.
                                      }

                                      stream_mode_toggle.on_hover_text("Continuously retrieve messages delivered to this subscription.");
                                });

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for message in subscription.messages.iter() {
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

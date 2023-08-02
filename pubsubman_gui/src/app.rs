use std::collections::HashMap;

use pubsubman_backend::{
    message::{BackendMessage, FrontendMessage},
    Backend,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{subscription::Subscription, topic::Topic};

pub struct TemplateApp {
    topics: Vec<Topic>,
    selected_topic: Option<String>,
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
            selected_topic: None,
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
                            .selected_topic
                            .as_ref()
                            .is_some_and(|id| *id == topic.id);

                        let on_click = || {
                            self.selected_topic = Some(topic.id.to_string());

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

    fn render_central_panel(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.selected_topic {
                Some(topic_id) => {
                    egui::TopBottomPanel::top("topic_view_top_panel").show_inside(ui, |ui| {
                        ui.heading(topic_id);
                    });

                    match self.subscriptions.get(topic_id) {
                        Some(subscription) => {
                            egui::TopBottomPanel::bottom("topic_view_bottom_panel")
                                .exact_height(250.0)
                                .show_inside(ui, |ui| {
                                    ui.heading("Publish");
                                });

                            egui::CentralPanel::default().show_inside(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.heading("Messages");
                                    if ui.button("Pull").clicked() {
                                        let front_tx = self.front_tx.clone();
                                        let sub_id = subscription.id.clone();

                                        tokio::spawn(async move {
                                            front_tx
                                                .send(FrontendMessage::PullMessages(sub_id))
                                                .await
                                                .unwrap();
                                        });
                                    }
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
                                                        ui.label(publish_time.to_string());
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

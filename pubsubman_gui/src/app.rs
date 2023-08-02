use std::collections::HashMap;

use pubsubman_backend::{
    message::{BackendMessage, FrontendMessage},
    Backend,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::topic::Topic;

pub struct TemplateApp {
    topics: Vec<Topic>,
    selected_topic: Option<String>,
    /// The subscriptions this app has created in order to recieve messages.
    /// Mapping of topic ID -> subscription ID.
    subscriptions: HashMap<String, String>,
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
                    self.subscriptions.insert(topic_id, sub_id);
                }
            },
            Err(err) => println!("{:?}", err),
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
                    ui.heading(topic_id);

                    match self.subscriptions.get(topic_id) {
                        Some(sub_id) => {
                            ui.heading(format!("Active subscription: {}", sub_id));
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

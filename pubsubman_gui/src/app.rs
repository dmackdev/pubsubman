use std::collections::HashMap;

use pubsubman_backend::{
    message::{BackendMessage, FrontendMessage},
    model::{PubsubMessage, SubscriptionName, TopicName},
    Backend,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    actions::{create_subscription, refresh_topics},
    ui::{render_topic_name, MessagesView, PublishView},
};

pub struct App {
    topic_names: Vec<TopicName>,
    selected_topic: Option<TopicName>,
    /// The subscriptions this app has created in order to recieve messages.
    subscriptions: HashMap<TopicName, SubscriptionName>,
    messages: HashMap<TopicName, Vec<PubsubMessage>>,
    publish_views: HashMap<TopicName, PublishView>,
    messages_views: HashMap<TopicName, MessagesView>,
    front_tx: Sender<FrontendMessage>,
    back_rx: Receiver<BackendMessage>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (front_tx, front_rx) = tokio::sync::mpsc::channel(10);
        let (back_tx, back_rx) = tokio::sync::mpsc::channel(10);

        tokio::spawn(async move {
            Backend::new(back_tx, front_rx).await.init();
        });

        refresh_topics(&front_tx);

        Self {
            topic_names: vec![],
            selected_topic: None,
            subscriptions: HashMap::new(),
            messages: HashMap::new(),
            publish_views: HashMap::new(),
            messages_views: HashMap::new(),
            front_tx,
            back_rx,
        }
    }

    fn handle_backend_message(&mut self) {
        match self.back_rx.try_recv() {
            Ok(message) => match message {
                BackendMessage::TopicsUpdated(topic_names) => {
                    self.topic_names = topic_names;
                    refresh_topics(&self.front_tx);
                }
                BackendMessage::SubscriptionCreated(topic_name, sub_name) => {
                    self.subscriptions.insert(topic_name, sub_name);
                }
                BackendMessage::MessageReceived(topic_name, message) => {
                    self.messages.entry(topic_name).or_default().push(message);
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

                    let topic_names = self.topic_names.clone();
                    for topic_name in topic_names {
                        let is_selected = self.is_topic_selected(&topic_name);

                        render_topic_name(ui, &topic_name, is_selected, || {
                            self.on_topic_click(&topic_name)
                        });
                    }
                });
            });
    }

    fn on_topic_click(&mut self, topic_name: &TopicName) {
        if self.is_topic_selected(topic_name) {
            return;
        }

        if let Some(cancel_token) = self
            .selected_topic
            .take()
            .and_then(|selected_topic| self.messages_views.get_mut(&selected_topic))
            .and_then(|messages_view| {
                messages_view.stream_messages_enabled = false;
                messages_view.stream_messages_cancel_token.take()
            })
        {
            cancel_token.cancel();
        }

        self.selected_topic = Some(topic_name.clone());

        if !self.subscriptions.contains_key(topic_name) {
            create_subscription(&self.front_tx, topic_name);
        }
    }

    fn is_topic_selected(&self, topic_name: &TopicName) -> bool {
        self.selected_topic
            .as_ref()
            .is_some_and(|selected_topic| selected_topic == topic_name)
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.selected_topic {
                Some(selected_topic) => {
                    egui::TopBottomPanel::top("topic_view_top_panel").show_inside(ui, |ui| {
                        ui.heading(&selected_topic.0);
                    });

                    egui::TopBottomPanel::bottom("topic_view_bottom_panel")
                        .exact_height(250.0)
                        .show_inside(ui, |ui| {
                            self.publish_views
                                .entry(selected_topic.clone())
                                .or_default()
                                .show(ui, &self.front_tx, selected_topic);
                        });

                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        match self.subscriptions.get(selected_topic) {
                            Some(sub_name) => {
                                let messages_view = self
                                    .messages_views
                                    .entry(selected_topic.clone())
                                    .or_default();

                                messages_view.show(
                                    ui,
                                    &self.front_tx,
                                    selected_topic,
                                    sub_name,
                                    self.messages.get(selected_topic).unwrap_or(&vec![]),
                                );
                            }
                            None => {
                                ui.vertical_centered(|ui| {
                                    ui.allocate_space(ui.available_size() / 2.0);
                                    ui.spinner();
                                });
                            }
                        }
                    });
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.handle_backend_message();
        self.render_top_panel(ctx, frame);
        self.render_topics_panel(ctx);
        self.render_central_panel(ctx);
    }
}

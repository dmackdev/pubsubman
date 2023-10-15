use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local};
use egui_json_tree::{DefaultExpand, JsonTree};
use pubsubman_backend::{
    message::{BackendMessage, FrontendMessage},
    model::{PubsubMessage, PubsubMessageToPublish, SubscriptionName, TopicName},
    Backend,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    actions::{create_subscription, delete_subscriptions, publish_message, refresh_topics},
    column_settings::ColumnSettings,
    exit_state::{ExitState, SubscriptionCleanupState},
    notifications::Notifications,
    settings::Settings,
    ui::{render_topic_name, show_json_context_menu, MessagesView, PublishView},
};

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Memory {
    pub messages: HashMap<TopicName, Vec<PubsubMessage>>,
    /// The subscriptions this app has created in order to recieve messages.
    subscriptions: HashMap<TopicName, SubscriptionName>,
    pub column_settings: HashMap<TopicName, ColumnSettings>,
    pub settings: Settings,
}

pub struct App {
    topic_names: Vec<TopicName>,
    selected_topic: Option<TopicName>,
    publish_views: HashMap<TopicName, PublishView>,
    messages_views: HashMap<TopicName, MessagesView>,
    exit_state: ExitState,
    memory: Memory,
    front_tx: Sender<FrontendMessage>,
    back_rx: Receiver<BackendMessage>,
    notifications: Notifications,
    selected_message: Option<(TopicName, usize)>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, emulator_project_id: Option<String>) -> Self {
        let (front_tx, front_rx) = tokio::sync::mpsc::channel(10);
        let (back_tx, back_rx) = tokio::sync::mpsc::channel(10);

        std::thread::spawn(|| {
            if let Ok(mut backend) = Backend::new(back_tx, front_rx, emulator_project_id) {
                backend.init();
            };
        });

        refresh_topics(&front_tx, None);

        let memory = cc
            .storage
            .and_then(|storage| eframe::get_value::<Memory>(storage, eframe::APP_KEY))
            .unwrap_or_default();

        Self {
            topic_names: vec![],
            selected_topic: None,
            publish_views: HashMap::default(),
            messages_views: HashMap::default(),
            exit_state: ExitState::default(),
            memory,
            front_tx,
            back_rx,
            notifications: Notifications::default(),
            selected_message: None,
        }
    }

    fn handle_backend_message(&mut self) {
        match self.back_rx.try_recv() {
            Ok(message) => match message {
                BackendMessage::ClientInitialised(project_id) => self
                    .notifications
                    .success(format!("Successfully authenticated to: {}.", project_id)),
                BackendMessage::TopicsUpdated(topic_names) => {
                    self.topic_names = topic_names;

                    refresh_topics(&self.front_tx, Some(5000));
                }
                BackendMessage::SubscriptionCreated(topic_name, sub_name) => {
                    self.memory.subscriptions.insert(topic_name, sub_name);
                }
                BackendMessage::MessageReceived(topic_name, message) => {
                    self.memory
                        .messages
                        .entry(topic_name)
                        .or_default()
                        .push(message);
                }
                BackendMessage::SubscriptionsDeleted(results) => {
                    let successfully_deleted: HashSet<SubscriptionName> =
                        results.into_iter().filter_map(|s| s.ok()).collect();

                    self.memory
                        .subscriptions
                        .retain(|_, sub_name| !successfully_deleted.contains(sub_name));

                    self.exit_state.subscription_cleanup_state = SubscriptionCleanupState::Complete;
                }
                BackendMessage::Error(err) => self.notifications.error(err),
            },
            Err(_err) => {}
        }
    }

    fn render_top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.close_menu();
                        frame.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.memory.settings.view.show_publish_message_panel,
                            " Publish Message Panel",
                        );
                    });
                });
            });
        });
    }

    fn render_topics_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .resizable(true)
            .default_width(250.0)
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

        self.selected_message.take();

        self.selected_topic = Some(topic_name.clone());

        if !self.memory.subscriptions.contains_key(topic_name) {
            create_subscription(&self.front_tx, topic_name);
        }
    }

    fn is_topic_selected(&self, topic_name: &TopicName) -> bool {
        self.selected_topic
            .as_ref()
            .is_some_and(|selected_topic| selected_topic == topic_name)
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        match &self.selected_topic {
            Some(selected_topic) => {
                let selected_message =
                    &self
                        .selected_message
                        .as_ref()
                        .and_then(|(topic_name, idx)| {
                            self.memory
                                .messages
                                .get(topic_name)
                                .and_then(|messages| messages.get(*idx))
                        });

                egui::SidePanel::right("selected_message")
                    .frame(egui::Frame::none())
                    .resizable(true)
                    .show_animated(ctx, selected_message.is_some(), |ui| {
                        if let Some(message) = selected_message {
                            egui::TopBottomPanel::top("selected_message_top_panel")
                                .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                                .show_inside(ui, |ui| {
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.heading(format!("Message {}", &message.id));
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("✖").clicked() {
                                                        self.selected_message.take();
                                                    }
                                                },
                                            );
                                        },
                                    );
                                });

                            egui::TopBottomPanel::bottom("selected_message_bottom_panel")
                                .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                                .show_inside(ui, |ui| {
                                    if ui.button("Republish Message").clicked() {
                                        let message_to_publish = PubsubMessageToPublish::new(
                                            message.data.clone(),
                                            message.attributes.clone(),
                                        );
                                        publish_message(
                                            &self.front_tx,
                                            selected_topic,
                                            message_to_publish,
                                        )
                                    }
                                });

                            egui::CentralPanel::default().show_inside(ui, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    let publish_time =
                                        if let Some(publish_time) = message.publish_time {
                                            let local_publish_time: DateTime<Local> =
                                                publish_time.into();
                                            local_publish_time.format("%d/%m/%Y %H:%M").to_string()
                                        } else {
                                            "<Empty>".to_string()
                                        };

                                    ui.horizontal(|ui| {
                                        ui.label("Publish Time: ");
                                        ui.monospace(publish_time);
                                    });

                                    egui::CollapsingHeader::new("Data")
                                        .id_source("selected_message_data_collapsing_header")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            JsonTree::new(
                                                format!(
                                                    "selected_message_data_json_{}",
                                                    &message.id
                                                ),
                                                &message.data_json,
                                            )
                                            .default_expand(DefaultExpand::All)
                                            .response_callback(show_json_context_menu(
                                                &message.data_json,
                                            ))
                                            .show(ui);
                                        });

                                    egui::CollapsingHeader::new("Attributes")
                                        .id_source("selected_message_attributes_collapsing_header")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            if message.attributes.is_empty() {
                                                ui.monospace("<Empty>");
                                            } else {
                                                JsonTree::new(
                                                    format!(
                                                        "selected_message_attributes_json_{}",
                                                        &message.id
                                                    ),
                                                    &message.attributes_json,
                                                )
                                                .default_expand(egui_json_tree::DefaultExpand::All)
                                                .response_callback(show_json_context_menu(
                                                    &message.attributes_json,
                                                ))
                                                .show(ui);
                                            }
                                        });
                                    ui.allocate_space(ui.available_size());
                                });
                            });
                        }
                    });

                egui::TopBottomPanel::top("topic_view_top_panel")
                    .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(&selected_topic.0);
                        });
                    });

                egui::TopBottomPanel::bottom("topic_view_bottom_panel")
                    .resizable(true)
                    .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                    .show_animated(
                        ctx,
                        self.memory.settings.view.show_publish_message_panel,
                        |ui| {
                            self.publish_views
                                .entry(selected_topic.clone())
                                .or_default()
                                .show(ui, &self.front_tx, selected_topic);

                            ui.allocate_space(ui.available_size());
                        },
                    );

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::central_panel(&ctx.style())
                            .inner_margin(0.0)
                            .outer_margin(0.0),
                    )
                    .show(ctx, |ui| {
                        match self.memory.subscriptions.get(selected_topic) {
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
                                    self.memory
                                        .column_settings
                                        .entry(selected_topic.clone())
                                        .or_default(),
                                    self.memory.messages.get(selected_topic).unwrap_or(&vec![]),
                                    |idx| {
                                        self.selected_message = Some((selected_topic.clone(), idx))
                                    },
                                );
                            }
                            None => {
                                ui.with_layout(
                                    egui::Layout::centered_and_justified(
                                        egui::Direction::LeftToRight,
                                    ),
                                    |ui| {
                                        ui.spinner();
                                    },
                                );
                            }
                        }
                    });
            }
            None => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.heading("Select a Topic.");
                        },
                    );
                });
            }
        };
    }

    fn render_close_dialog(&mut self, ctx: &egui::Context) {
        let sub_names = self.memory.subscriptions.values().cloned().collect();
        let cleanup_subscriptions = |sub_names: Vec<SubscriptionName>| {
            delete_subscriptions(&self.front_tx, sub_names);
        };
        self.exit_state.show(ctx, sub_names, cleanup_subscriptions)
    }

    fn handle_exit(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.exit_state.can_exit {
            for view in self.messages_views.values_mut() {
                if let Some(cancel_token) = view.stream_messages_cancel_token.take() {
                    cancel_token.cancel();
                }
            }
            // Clear superficial widget state, e.g. reset all collapsing headers.
            ctx.data_mut(|d| d.clear());
            frame.close();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.handle_backend_message();
        self.render_close_dialog(ctx);
        self.render_top_panel(ctx, frame);
        self.render_topics_panel(ctx);
        self.render_central_panel(ctx);
        self.handle_exit(ctx, frame);
        self.notifications.show(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value::<Memory>(storage, eframe::APP_KEY, &self.memory);
    }

    fn on_close_event(&mut self) -> bool {
        self.exit_state.on_close_event()
    }
}

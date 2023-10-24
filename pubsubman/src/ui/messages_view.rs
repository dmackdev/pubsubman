use std::fmt::Display;

use chrono::{DateTime, Local};
use egui_json_tree::{DefaultExpand, JsonTree};
use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessage, SubscriptionName, TopicName},
};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{
    actions::{pull_message_batch, stream_messages},
    column_settings::ColumnSettings,
};

use super::show_json_context_menu;

#[derive(Default)]
pub struct MessagesView {
    pub stream_messages_enabled: bool,
    pub stream_messages_cancel_token: Option<CancellationToken>,
    pub search_query: String,
    search_mode: SearchMode,
}

impl MessagesView {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        front_tx: &Sender<FrontendMessage>,
        selected_topic: &TopicName,
        sub_name: &SubscriptionName,
        column_settings: &mut ColumnSettings,
        messages: &[PubsubMessage],
        on_message_id_click: impl FnMut(usize),
    ) {
        let search_query = self.search_query.to_ascii_lowercase();
        let search_mode = self.search_mode;
        let filtered_messages = messages.iter().filter(|msg| {
            let source = match search_mode {
                SearchMode::Data => &msg.data,
                SearchMode::Id => &msg.id,
            };
            source.to_ascii_lowercase().contains(&search_query)
        });

        egui::TopBottomPanel::top("messages_top_panel")
            .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(8.0))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Messages");

                    ui.add_enabled_ui(!self.stream_messages_enabled, |ui| {
                        let pull_button = ui.button("Pull");
                        if pull_button.clicked() {
                            pull_message_batch(
                                front_tx,
                                selected_topic,
                                sub_name,
                                &CancellationToken::new(),
                            );
                        }
                        pull_button
                            .on_hover_text(
                                "Retrieve batch of all undelivered messages on this subscription.",
                            )
                            .on_disabled_hover_text("Disable Stream mode to Pull messages.");
                    });

                    let stream_mode_toggle = ui.add(
                        egui::Button::new("Stream")
                            .selected(self.stream_messages_enabled)
                            .rounding(ui.visuals().widgets.active.rounding),
                    );

                    if stream_mode_toggle.clicked() {
                        self.stream_messages_enabled = !self.stream_messages_enabled;

                        if self.stream_messages_enabled {
                            let cancel_token = CancellationToken::new();

                            stream_messages(front_tx, selected_topic, sub_name, &cancel_token);

                            self.stream_messages_cancel_token = Some(cancel_token);
                        } else if let Some(cancel_token) = self.stream_messages_cancel_token.take()
                        {
                            cancel_token.cancel();
                        }
                    }

                    stream_mode_toggle.on_hover_text(
                        "Continuously retrieve messages delivered to this subscription.",
                    );
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(ui.style())
                    .fill(ui.style().visuals.extreme_bg_color)
                    .inner_margin(egui::vec2(16.0, 12.0)),
            )
            .show_inside(ui, |ui| {
                if messages.is_empty() {
                    ui.allocate_space(ui.available_size() / 2.0);
                    ui.vertical_centered(|ui| {
                        ui.heading("No messages received for this Topic.");
                        ui.label("Pull or Stream new messages to retrieve the latest.");
                    });
                } else {
                    let mut should_reset_expanded = false;

                    ui.horizontal(|ui| {
                        ui.visuals_mut().extreme_bg_color = egui::Color32::from_gray(32);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            let search_query_edit_response = ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .desired_width(125.0)
                                    .hint_text("Search"),
                            );

                            if search_query_edit_response.changed() {
                                should_reset_expanded = true;
                            }

                            let search_mode_changed =
                                egui::ComboBox::from_id_source("search_mode_combo_box")
                                    .selected_text(format!("{}", self.search_mode))
                                    .width(50.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.search_mode,
                                            SearchMode::Data,
                                            "Data",
                                        )
                                        .changed()
                                            || ui
                                                .selectable_value(
                                                    &mut self.search_mode,
                                                    SearchMode::Id,
                                                    "ID",
                                                )
                                                .changed()
                                    })
                                    .inner
                                    .unwrap_or_default();

                            if search_mode_changed {
                                should_reset_expanded = true;
                            }

                            ui.visuals_mut().widgets.inactive.weak_bg_fill =
                                egui::Color32::from_gray(32);

                            if ui.button("✖").clicked() && !self.search_query.is_empty() {
                                self.search_query.clear();
                                should_reset_expanded = true;
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| column_settings.show(ui),
                            );
                        });
                    });

                    let outer_margin = egui::Margin {
                        top: 8.0,
                        bottom: 12.0,
                        ..Default::default()
                    };

                    egui::Frame::none()
                        .fill(ui.style().visuals.panel_fill)
                        .inner_margin(egui::vec2(6.0, 3.0))
                        .outer_margin(outer_margin)
                        .rounding(ui.style().visuals.window_rounding)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .stick_to_bottom(true)
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    render_messages_table(
                                        ui,
                                        selected_topic,
                                        column_settings,
                                        filtered_messages,
                                        &search_query,
                                        should_reset_expanded,
                                        self.search_mode,
                                        on_message_id_click,
                                    );
                                });
                        });
                }
            });
    }
}

#[allow(clippy::too_many_arguments)]
fn render_messages_table<'a, I>(
    ui: &mut egui::Ui,
    selected_topic: &TopicName,
    column_settings: &ColumnSettings,
    messages: I,
    search_term: &str,
    should_reset_expanded: bool,
    search_mode: SearchMode,
    mut on_message_id_click: impl FnMut(usize),
) where
    I: Iterator<Item = &'a PubsubMessage>,
{
    let ColumnSettings { show_published_at } = *column_settings;

    let mut num_columns = 2; // ID and Data columns will always be shown.

    if show_published_at {
        num_columns += 1;
    }

    egui::Grid::new(&selected_topic.0)
        .striped(true)
        .num_columns(num_columns)
        .spacing((25.0, 8.0))
        .show(ui, |ui| {
            ui.label("ID");

            if show_published_at {
                ui.label("Published at");
            }

            // Let Data column take up all remaining space.
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Center)
                    .with_main_align(egui::Align::LEFT)
                    .with_main_justify(true),
                |ui| {
                    ui.label("Data");
                },
            );

            ui.end_row();

            for (idx, message) in messages.enumerate() {
                if ui.link(&message.id).clicked() {
                    on_message_id_click(idx);
                }

                if show_published_at {
                    if let Some(publish_time) = message.publish_time {
                        let local_publish_time: DateTime<Local> = publish_time.into();

                        ui.monospace(format!("{}", local_publish_time.format("%d/%m/%Y %H:%M")));
                    }
                }

                let default_expand = match search_mode {
                    SearchMode::Data => DefaultExpand::SearchResults(search_term),
                    SearchMode::Id => DefaultExpand::None,
                };

                let response = JsonTree::new(&message.id, &message.data_json)
                    .default_expand(default_expand)
                    .response_callback(show_json_context_menu(&message.data_json))
                    .show(ui);

                if should_reset_expanded {
                    response.reset_expanded(ui);
                }

                ui.end_row();
            }
        });
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum SearchMode {
    #[default]
    Data,
    Id,
}

impl Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::Data => write!(f, "Data"),
            SearchMode::Id => write!(f, "ID"),
        }
    }
}

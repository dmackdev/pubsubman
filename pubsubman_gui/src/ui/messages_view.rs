use std::collections::HashMap;

use chrono::{DateTime, Local};
use egui_json_tree::JsonTree;
use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessage, SubscriptionName, TopicName},
};
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{
    actions::{pull_message_batch, stream_messages},
    column_settings::ColumnSettings,
};

#[derive(Default)]
pub struct MessagesView {
    pub stream_messages_enabled: bool,
    pub stream_messages_cancel_token: Option<CancellationToken>,
    pub search_query: String,
}

impl MessagesView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        front_tx: &Sender<FrontendMessage>,
        selected_topic: &TopicName,
        sub_name: &SubscriptionName,
        column_settings: &mut ColumnSettings,
        messages: &[PubsubMessage],
    ) {
        let search_query = self.search_query.to_ascii_lowercase();
        let filtered_messages = messages
            .iter()
            .filter(|msg| msg.data.to_ascii_lowercase().contains(&search_query));

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

                    let stream_mode_toggle =
                        ui.toggle_value(&mut self.stream_messages_enabled, "Stream");

                    if stream_mode_toggle.changed() {
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
                    ui.horizontal(|ui| {
                        ui.visuals_mut().extreme_bg_color = egui::Color32::from_gray(32);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .desired_width(125.0)
                                    .hint_text("Search"),
                            );

                            ui.visuals_mut().widgets.inactive.weak_bg_fill =
                                egui::Color32::from_gray(32);
                            if ui.button("✖").clicked() {
                                self.search_query.clear();
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
                            render_messages_table(
                                ui,
                                column_settings,
                                filtered_messages,
                                &search_query,
                            );
                        });
                }
            });
    }
}

fn render_messages_table<'a, I>(
    ui: &mut egui::Ui,
    column_settings: &ColumnSettings,
    messages: I,
    search_term: &str,
) where
    I: Iterator<Item = &'a PubsubMessage>,
{
    let ColumnSettings {
        show_id,
        show_published_at,
        show_attributes,
    } = *column_settings;

    let num_columns = [show_id, show_published_at, show_attributes].iter().fold(
        1, // Data column will always be present
        |acc, col_enabled| if *col_enabled { acc + 1 } else { acc },
    );

    egui::Grid::new("some_unique_id")
        .striped(true)
        .num_columns(num_columns)
        .show(ui, |ui| {
            if show_id {
                ui.label("ID");
            }

            if show_published_at {
                ui.label("Published at");
            }

            if show_attributes {
                ui.label("Attributes");
            }

            ui.label("Data");

            ui.end_row();

            for message in messages {
                if show_id {
                    ui.label(&message.id);
                }

                if show_published_at {
                    if let Some(publish_time) = message.publish_time {
                        let local_publish_time: DateTime<Local> = publish_time.into();

                        ui.label(format!("{}", local_publish_time.format("%d/%m/%Y %H:%M")));
                    }
                }

                if show_attributes {
                    ui.label(format_attributes(&message.attributes));
                }

                let value: Value = match serde_json::from_str(&message.data) {
                    Ok(val) => val,
                    Err(_) => Value::String(message.data.clone()),
                };

                JsonTree::new(&message.id, &value)
                    .default_expand(egui_json_tree::Expand::SearchResults(
                        search_term.to_string(),
                    ))
                    .show(ui);

                ui.end_row();
            }
        });
}

fn format_attributes(attributes: &HashMap<String, String>) -> String {
    attributes
        .iter()
        .enumerate()
        .map(|(i, (k, v))| {
            format!(
                "{}:{}{}",
                k,
                v,
                (if i == attributes.len() - 1 { "" } else { ", " })
            )
        })
        .collect()
}

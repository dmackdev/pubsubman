use std::collections::HashMap;

use chrono::{DateTime, Local};
use egui_extras::{Column, TableBuilder};
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
                            render_messages_table(ui, column_settings, filtered_messages);
                        });
                }
            });
    }
}

const ROW_HEIGHT_PADDING: f32 = 4.0;

fn render_messages_table<'a, I>(ui: &mut egui::Ui, column_settings: &ColumnSettings, messages: I)
where
    I: Iterator<Item = &'a PubsubMessage>,
{
    let text_height = ui.text_style_height(&egui::TextStyle::Monospace);

    let mut table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(0.0)
        .auto_shrink([false, true])
        .stick_to_bottom(true);

    let ColumnSettings {
        show_id,
        show_published_at,
        show_attributes,
    } = *column_settings;

    for col_enabled in [show_id, show_published_at, show_attributes] {
        if col_enabled {
            table = table.column(Column::auto());
        }
    }

    // For Data column which cannot be hidden
    table = table.column(Column::remainder());

    table
        .header(20.0, |mut header| {
            if show_id {
                header.col(|ui| {
                    ui.label("ID");
                });
            }

            if show_published_at {
                header.col(|ui| {
                    ui.label("Published at");
                });
            }

            if show_attributes {
                header.col(|ui| {
                    ui.label("Attributes");
                });
            }

            header.col(|ui| {
                ui.label("Data");
            });
        })
        .body(|mut body| {
            for message in messages {
                let num_lines = message.data.split('\n').count();
                let row_height = num_lines as f32 * text_height + ROW_HEIGHT_PADDING;

                body.row(row_height, |mut row| {
                    if show_id {
                        row.col(|ui| {
                            ui.label(&message.id);
                        });
                    }

                    if show_published_at {
                        row.col(|ui| {
                            if let Some(publish_time) = message.publish_time {
                                let local_publish_time: DateTime<Local> = publish_time.into();

                                ui.label(format!(
                                    "{}",
                                    local_publish_time.format("%d/%m/%Y %H:%M")
                                ));
                            }
                        });
                    }

                    if show_attributes {
                        row.col(|ui| {
                            ui.label(format_attributes(&message.attributes));
                        });
                    }

                    row.col(|ui| {
                        ui.label(&message.data);
                    });
                });
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

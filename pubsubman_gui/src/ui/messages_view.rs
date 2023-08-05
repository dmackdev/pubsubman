use std::collections::HashMap;

use chrono::{DateTime, Local};
use egui_extras::{Column, TableBuilder};
use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessage, SubscriptionName, TopicName},
};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::actions::{pull_message_batch, stream_messages};

#[derive(Default)]
pub struct MessagesView {
    pub stream_messages_enabled: bool,
    pub stream_messages_cancel_token: Option<CancellationToken>,
}

impl MessagesView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        front_tx: &Sender<FrontendMessage>,
        selected_topic: &TopicName,
        sub_name: &SubscriptionName,
        messages: &[PubsubMessage],
    ) {
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
                    .inner_margin(16.0),
            )
            .show_inside(ui, |ui| {
                if messages.is_empty() {
                    ui.allocate_space(ui.available_size() / 2.0);
                    ui.vertical_centered(|ui| {
                        ui.heading("No messages received for this Topic.");
                        ui.label("Pull or Stream new messages to retrieve the latest.");
                    });
                } else {
                    egui::Frame::none()
                        .fill(ui.style().visuals.panel_fill)
                        .inner_margin(egui::Vec2::new(6.0, 3.0))
                        .rounding(ui.style().visuals.window_rounding)
                        .show(ui, |ui| {
                            render_messages_table(ui, messages);
                        });
                }
            });
    }
}

const ROW_HEIGHT_PADDING: f32 = 4.0;

fn render_messages_table(ui: &mut egui::Ui, messages: &[PubsubMessage]) {
    let text_height = ui.text_style_height(&egui::TextStyle::Monospace);

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::remainder())
        .min_scrolled_height(0.0)
        .auto_shrink([false, true]);

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("ID");
            });

            header.col(|ui| {
                ui.label("Published at");
            });

            header.col(|ui| {
                ui.label("Attributes");
            });

            header.col(|ui| {
                ui.label("Payload");
            });
        })
        .body(|mut body| {
            for message in messages {
                let num_lines = message.data.split('\n').count();
                let row_height = num_lines as f32 * text_height + ROW_HEIGHT_PADDING;

                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(&message.id);
                    });

                    row.col(|ui| {
                        if let Some(publish_time) = message.publish_time {
                            let local_publish_time: DateTime<Local> = publish_time.into();

                            ui.label(format!("{}", local_publish_time.format("%d/%m/%Y %H:%M")));
                        }
                    });

                    row.col(|ui| {
                        ui.label(format_attributes(&message.attributes));
                    });

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

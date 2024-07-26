use chrono::{DateTime, Local};
use egui_json_tree::{DefaultExpand, JsonTree};
use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessage, PubsubMessageToPublish, TopicName},
};
use tokio::sync::mpsc::Sender;

use crate::actions::publish_message;

use super::show_json_context_menu;

pub fn render_selected_message(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    front_tx: &Sender<FrontendMessage>,
    message: &PubsubMessage,
    selected_topic: &TopicName,
    mut on_close: impl FnMut(),
) {
    egui::TopBottomPanel::top("selected_message_top_panel")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
        .show_inside(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.heading(format!("Message {}", &message.id));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        on_close();
                    }
                });
            });
        });

    egui::TopBottomPanel::bottom("selected_message_bottom_panel")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
        .show_inside(ui, |ui| {
            if ui.button("Republish Message").clicked() {
                let message_to_publish =
                    PubsubMessageToPublish::new(message.data.clone(), message.attributes.clone());
                publish_message(front_tx, selected_topic, message_to_publish)
            }
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let publish_time = if let Some(publish_time) = message.publish_time {
                let local_publish_time: DateTime<Local> = publish_time.into();
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
                        format!("selected_message_data_json_{}", &message.id),
                        &message.data_json,
                    )
                    .default_expand(DefaultExpand::All)
                    .on_render(show_json_context_menu)
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
                            format!("selected_message_attributes_json_{}", &message.id),
                            &message.attributes_json,
                        )
                        .default_expand(egui_json_tree::DefaultExpand::All)
                        .on_render(show_json_context_menu)
                        .show(ui);
                    }
                });
            ui.allocate_space(ui.available_size());
        });
    });
}

use std::collections::HashMap;

use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessageToPublish, TopicName},
};
use tokio::sync::mpsc::Sender;

use crate::actions::publish_message;

#[derive(Default)]
pub struct PublishView {
    data: String,
    attributes: Vec<(String, String)>,
}

impl PublishView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        front_tx: &Sender<FrontendMessage>,
        selected_topic: &TopicName,
    ) {
        ui.heading("New Message");

        egui::CollapsingHeader::new("Data")
            .id_source(format!("{}-data", selected_topic.0))
            .default_open(true)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.data)
                        .code_editor()
                        .desired_rows(4)
                        .desired_width(250.0),
                );
            });

        egui::CollapsingHeader::new("Attributes")
            .id_source(format!("{}-attributes", selected_topic.0))
            .default_open(false)
            .show(ui, |ui| {
                let mut attr_idx_to_delete = None;

                if !self.attributes.is_empty() {
                    let table = egui_extras::TableBuilder::new(ui)
                        .striped(false)
                        .cell_layout(egui::Layout::centered_and_justified(
                            egui::Direction::LeftToRight,
                        ))
                        .column(egui_extras::Column::exact(100.0))
                        .column(egui_extras::Column::exact(100.0))
                        .column(egui_extras::Column::auto());

                    table.body(|mut body| {
                        for (idx, (id, val)) in self.attributes.iter_mut().enumerate() {
                            body.row(18.0, |mut row| {
                                row.col(|ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(id)
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .hint_text("Key"),
                                    );
                                });

                                row.col(|ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(val)
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .hint_text("Value"),
                                    );
                                });

                                row.col(|ui| {
                                    if ui.button("✖").clicked() {
                                        attr_idx_to_delete = Some(idx);
                                    }
                                });
                            });
                        }
                    });
                }

                if let Some(i) = attr_idx_to_delete {
                    self.attributes.remove(i);
                }

                if ui.button("➕").clicked() {
                    self.attributes.push(("".to_string(), "".to_string()));
                }
            });

        if ui.button("Publish").clicked() {
            publish_message(front_tx, selected_topic, self.into())
        }
    }
}

impl From<&mut PublishView> for PubsubMessageToPublish {
    fn from(val: &mut PublishView) -> Self {
        Self::new(
            val.data.clone(),
            HashMap::from_iter(val.attributes.clone().into_iter()),
        )
    }
}

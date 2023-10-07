use std::collections::{HashMap, HashSet};

use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessageToPublish, TopicName},
};
use tokio::sync::mpsc::Sender;

use crate::actions::publish_message;

#[derive(Default)]
pub struct PublishView {
    data: String,
    attributes: Attributes,
}

impl PublishView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        front_tx: &Sender<FrontendMessage>,
        selected_topic: &TopicName,
    ) {
        ui.heading("Publish New Message");

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
                if !self.attributes.is_empty() {
                    egui::Grid::new(format!("{}-attributes-form", selected_topic.0))
                        .min_col_width(100.0)
                        .num_columns(3)
                        .spacing((0.0, 4.0))
                        .show(ui, |ui| {
                            self.attributes.show(ui);
                        });
                }

                if !self.attributes.is_empty() {
                    ui.add_space(4.0);
                }

                if ui.button("➕").clicked() {
                    self.attributes.push(("".to_string(), "".to_string()));
                }
            });

        ui.add_space(8.0);

        if ui.button("Publish").clicked() {
            publish_message(front_tx, selected_topic, self.into())
        }
    }
}

impl From<&mut PublishView> for PubsubMessageToPublish {
    fn from(val: &mut PublishView) -> Self {
        Self::new(
            val.data.clone(),
            HashMap::from_iter(val.attributes.0.clone()),
        )
    }
}

#[derive(Default)]
struct Attributes(Vec<(String, String)>);

impl Attributes {
    fn key_indices_map(&self) -> HashMap<String, HashSet<usize>> {
        let mut index_map = HashMap::new();

        for (index, (value, _)) in self.0.iter().enumerate() {
            index_map
                .entry(value.clone())
                .or_insert_with(HashSet::new)
                .insert(index);
        }

        index_map
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push(&mut self, attr: (String, String)) {
        self.0.push(attr);
    }

    fn is_valid(&self) -> bool {
        self.key_indices_map()
            .values()
            .all(|indices| indices.len() < 2)
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let mut attr_idx_to_delete = None;

        for (idx, (id, val)) in self.0.iter_mut().enumerate() {
            ui.add(
                egui::TextEdit::singleline(id)
                    .desired_width(100.0)
                    .code_editor()
                    .hint_text("Key"),
            );

            ui.add(
                egui::TextEdit::singleline(val)
                    .desired_width(100.0)
                    .code_editor()
                    .hint_text("Value"),
            );

            if ui.button("🗑").clicked() {
                attr_idx_to_delete = Some(idx);
            }

            ui.end_row();
        }

        if let Some(i) = attr_idx_to_delete {
            self.0.remove(i);
        }
    }
}

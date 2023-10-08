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

        let mut header_text = egui::RichText::new("Attributes");
        let attributes_key_count_map = self.attributes.key_count_map();
        let all_attributes_valid = attributes_key_count_map.iter().all(|(_, count)| *count < 2);

        if !all_attributes_valid {
            header_text = header_text.color(ui.visuals().error_fg_color);
        };

        egui::CollapsingHeader::new(header_text)
            .id_source(format!("{}-attributes", selected_topic.0))
            .default_open(false)
            .show(ui, |ui| {
                if !self.attributes.is_empty() {
                    egui::Grid::new(format!("{}-attributes-form", selected_topic.0))
                        .min_col_width(100.0)
                        .num_columns(3)
                        .spacing((0.0, 4.0))
                        .show(ui, |ui| {
                            self.attributes.show(ui, |key| {
                                attributes_key_count_map
                                    .get(key)
                                    .is_some_and(|count| *count < 2)
                            });
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

        if ui
            .add_enabled(all_attributes_valid, egui::Button::new("Publish"))
            .clicked()
        {
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
    fn key_count_map(&self) -> HashMap<String, usize> {
        let mut key_count_map = HashMap::new();

        for (key, _) in self.0.iter() {
            *key_count_map.entry(key.clone()).or_insert_with(|| 0) += 1;
        }

        key_count_map
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push(&mut self, attr: (String, String)) {
        self.0.push(attr);
    }

    fn show(&mut self, ui: &mut egui::Ui, is_key_valid: impl Fn(&str) -> bool) {
        let mut attr_idx_to_delete = None;

        for (idx, (key, val)) in self.0.iter_mut().enumerate() {
            let is_valid = is_key_valid(key);

            ui.validity_frame(is_valid).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(key)
                        .desired_width(100.0)
                        .code_editor()
                        .hint_text("Key"),
                );
            });

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

trait ValidityFrame {
    fn validity_frame(&self, is_valid: bool) -> egui::Frame;
}

impl ValidityFrame for &mut egui::Ui {
    fn validity_frame(&self, is_valid: bool) -> egui::Frame {
        let (stroke, rounding) = if is_valid {
            (egui::Stroke::NONE, egui::Rounding::ZERO)
        } else {
            (
                egui::Stroke {
                    width: 1.0,
                    color: self.visuals().error_fg_color,
                },
                self.visuals().widgets.hovered.rounding,
            )
        };

        egui::Frame::none()
            .stroke(stroke)
            .inner_margin(2.0)
            .rounding(rounding)
    }
}

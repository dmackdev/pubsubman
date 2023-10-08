use std::collections::HashMap;

use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessageToPublish, TopicName},
};
use tokio::sync::mpsc::Sender;

use crate::{actions::publish_message, ui::publish_view::attributes::AttributesKeyCounter};

use self::attributes::Attributes;

mod attributes;

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
        let attributes_key_count_map = key_count_map(ui.ctx(), &self.attributes);
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
        Self::new(val.data.clone(), (&val.attributes).into())
    }
}

fn key_count_map(ctx: &egui::Context, attributes: &Attributes) -> HashMap<String, usize> {
    impl egui::util::cache::ComputerMut<&Attributes, HashMap<String, usize>> for AttributesKeyCounter {
        fn compute(&mut self, attributes: &Attributes) -> HashMap<String, usize> {
            self.key_count_map(attributes)
        }
    }

    type AttributesKeyCounterCache =
        egui::util::cache::FrameCache<HashMap<String, usize>, AttributesKeyCounter>;

    ctx.memory_mut(|mem| {
        mem.caches
            .cache::<AttributesKeyCounterCache>()
            .get(attributes)
    })
}

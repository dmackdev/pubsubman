use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessageToPublish, TopicName},
};
use tokio::sync::mpsc::Sender;

use crate::actions::publish_message;

use self::attributes_form::{AttributesForm, attributes_validator};

mod attributes_form;

#[derive(Default)]
pub struct PublishView {
    data: String,
    attributes: AttributesForm,
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
            .id_salt(format!("{}-data", selected_topic.0))
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
        let attributes_validator = attributes_validator(ui.ctx(), &self.attributes);
        let all_attributes_valid = attributes_validator.is_valid();

        if !all_attributes_valid {
            header_text = header_text.color(ui.visuals().error_fg_color);
        };

        egui::CollapsingHeader::new(header_text)
            .id_salt(format!("{}-attributes", selected_topic.0))
            .default_open(false)
            .show(ui, |ui| {
                if !self.attributes.is_empty() {
                    egui::Grid::new(format!("{}-attributes-form", selected_topic.0))
                        .min_col_width(100.0)
                        .num_columns(3)
                        .spacing((0.0, 4.0))
                        .show(ui, |ui| {
                            self.attributes
                                .show(ui, |key| attributes_validator.is_key_valid(key));
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

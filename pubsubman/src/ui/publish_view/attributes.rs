use std::collections::HashMap;

use crate::ui::validity_frame::ValidityFrame;

#[derive(Default, Hash)]
pub struct Attributes(Vec<(String, String)>);

impl Attributes {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, attr: (String, String)) {
        self.0.push(attr);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, is_key_valid: impl Fn(&str) -> bool) {
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

impl From<&Attributes> for HashMap<String, String> {
    fn from(value: &Attributes) -> Self {
        HashMap::from_iter(value.0.clone())
    }
}

#[derive(Default)]
pub struct AttributesKeyCounter;

impl AttributesKeyCounter {
    pub fn key_count_map(&self, attributes: &Attributes) -> HashMap<String, usize> {
        let mut key_count_map = HashMap::new();

        for (key, _) in attributes.0.iter() {
            *key_count_map.entry(key.clone()).or_insert_with(|| 0) += 1;
        }

        key_count_map
    }
}

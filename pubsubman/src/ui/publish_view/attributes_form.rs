use std::collections::HashMap;

use crate::ui::validity_frame::ValidityFrame;

#[derive(Default, Hash)]
pub struct AttributesForm(Vec<(String, String)>);

impl AttributesForm {
    fn validator(&self) -> AttributesFormValidator {
        let mut key_count_map = HashMap::new();

        for (key, _) in self.0.iter() {
            *key_count_map.entry(key.clone()).or_insert_with(|| 0) += 1;
        }

        AttributesFormValidator(key_count_map)
    }

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

impl From<&AttributesForm> for HashMap<String, String> {
    fn from(value: &AttributesForm) -> Self {
        HashMap::from_iter(value.0.clone())
    }
}

#[derive(Default, Clone)]
pub struct AttributesFormValidator(HashMap<String, usize>);

impl AttributesFormValidator {
    pub fn is_valid(&self) -> bool {
        self.0.iter().all(|(_, count)| *count < 2)
    }

    pub fn is_key_valid(&self, key: &str) -> bool {
        self.0.get(key).is_some_and(|count| *count < 2)
    }
}

pub fn attributes_validator(
    ctx: &egui::Context,
    attributes: &AttributesForm,
) -> AttributesFormValidator {
    impl egui::util::cache::ComputerMut<&AttributesForm, AttributesFormValidator>
        for AttributesFormValidator
    {
        fn compute(&mut self, attributes: &AttributesForm) -> AttributesFormValidator {
            attributes.validator()
        }
    }

    type AttributesFormValidatorCache =
        egui::util::cache::FrameCache<AttributesFormValidator, AttributesFormValidator>;

    ctx.memory_mut(|mem| {
        mem.caches
            .cache::<AttributesFormValidatorCache>()
            .get(attributes)
    })
}

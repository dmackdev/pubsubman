#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ColumnSettings {
    pub show_publish_time: bool,
}

impl Default for ColumnSettings {
    fn default() -> Self {
        Self {
            show_publish_time: true,
        }
    }
}

impl ColumnSettings {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::from_gray(32);
        ui.menu_button("Columns ⏷", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_publish_time, " Publish Time");
            });
        });
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ColumnSettings {
    pub show_id: bool,
    pub show_published_at: bool,
    pub show_attributes: bool,
}

impl Default for ColumnSettings {
    fn default() -> Self {
        Self {
            show_id: true,
            show_published_at: true,
            show_attributes: true,
        }
    }
}

impl ColumnSettings {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::from_gray(32);
        ui.menu_button("Columns ⏷", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_id, " ID");
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_published_at, " Published at");
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_attributes, " Attributes");
            });
        });
    }
}

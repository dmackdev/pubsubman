#[derive(Default)]
pub struct PublishView {
    data: String,
}

impl PublishView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Publish");

        ui.label("Data:");
        ui.add(
            egui::TextEdit::multiline(&mut self.data)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .desired_rows(1)
                .lock_focus(true)
                .desired_width(f32::INFINITY),
        );
    }
}

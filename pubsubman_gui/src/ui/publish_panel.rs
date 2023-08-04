#[derive(Default)]
pub struct PublishMessageFormState {
    data: String,
}

impl PublishMessageFormState {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<()> {
        egui::TopBottomPanel::bottom("topic_view_bottom_panel")
            .exact_height(250.0)
            .show_inside(ui, |ui| {
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
            })
    }
}

#[derive(Debug)]
pub struct Topic {
    pub id: String,
}

impl Topic {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::none()
            .fill(ui.visuals().code_bg_color)
            .inner_margin(egui::Margin::same(5.0))
            .rounding(ui.visuals().window_rounding)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(&self.id);
            });

        let _response = frame
            .response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .interact(egui::Sense::click_and_drag());
    }
}

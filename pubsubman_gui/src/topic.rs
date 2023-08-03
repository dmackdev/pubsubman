use egui::style::Selection;

#[derive(Debug, Clone)]
pub struct Topic {
    pub id: String,
}

impl Topic {
    pub fn show(&self, ui: &mut egui::Ui, is_selected: bool, on_click: impl FnOnce()) {
        let (stroke, fill) = if is_selected {
            let Selection { stroke, bg_fill } = ui.visuals().selection;
            (stroke, bg_fill)
        } else {
            (egui::Stroke::NONE, ui.visuals().code_bg_color)
        };

        let frame = egui::Frame::none()
            .stroke(stroke)
            .fill(fill)
            .inner_margin(egui::Margin::same(7.5))
            .outer_margin(egui::Margin::same(2.5))
            .rounding(ui.visuals().window_rounding)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                let mut text = egui::RichText::new(&self.id);

                if is_selected {
                    text = text.color(stroke.color);
                }

                ui.label(text);
            });

        let response = frame
            .response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .interact(egui::Sense::click_and_drag());

        if response.clicked() {
            on_click();
        }
    }
}

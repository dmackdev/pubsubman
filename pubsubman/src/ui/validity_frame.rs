pub trait ValidityFrame {
    fn validity_frame(&self, is_valid: bool) -> egui::Frame;
}

impl ValidityFrame for &mut egui::Ui {
    fn validity_frame(&self, is_valid: bool) -> egui::Frame {
        let (stroke, rounding) = if is_valid {
            (egui::Stroke::NONE, egui::CornerRadius::ZERO)
        } else {
            (
                egui::Stroke {
                    width: 1.0,
                    color: self.visuals().error_fg_color,
                },
                self.visuals().widgets.hovered.corner_radius,
            )
        };

        egui::Frame::NONE
            .stroke(stroke)
            .inner_margin(2.0)
            .corner_radius(rounding)
    }
}

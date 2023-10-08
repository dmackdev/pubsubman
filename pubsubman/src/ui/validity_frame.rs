pub trait ValidityFrame {
    fn validity_frame(&self, is_valid: bool) -> egui::Frame;
}

impl ValidityFrame for &mut egui::Ui {
    fn validity_frame(&self, is_valid: bool) -> egui::Frame {
        let (stroke, rounding) = if is_valid {
            (egui::Stroke::NONE, egui::Rounding::ZERO)
        } else {
            (
                egui::Stroke {
                    width: 1.0,
                    color: self.visuals().error_fg_color,
                },
                self.visuals().widgets.hovered.rounding,
            )
        };

        egui::Frame::none()
            .stroke(stroke)
            .inner_margin(2.0)
            .rounding(rounding)
    }
}

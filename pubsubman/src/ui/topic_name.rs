use pubsubman_backend::model::TopicName;

pub fn render_topic_name(
    ui: &mut egui::Ui,
    topic_name: &TopicName,
    is_selected: bool,
    on_click: impl FnOnce(),
) {
    let (stroke, fill) = if is_selected {
        let egui::style::Selection { stroke, bg_fill } = ui.visuals().selection;
        (stroke, bg_fill)
    } else {
        (egui::Stroke::NONE, ui.visuals().code_bg_color)
    };

    egui::Frame::NONE
        .stroke(stroke)
        .fill(fill)
        .inner_margin(egui::Margin::same(7))
        .outer_margin(egui::Margin::same(2))
        .corner_radius(ui.visuals().window_corner_radius)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            let mut text = egui::RichText::new(&topic_name.0);

            if is_selected {
                text = text.color(stroke.color);
            }

            if ui
                .add(egui::Label::new(text).sense(egui::Sense::click()))
                .clicked()
            {
                on_click()
            }
        })
        .response
        .on_hover_cursor(egui::CursorIcon::PointingHand);
}

pub fn render_publish_panel(ui: &mut egui::Ui) -> egui::InnerResponse<()> {
    egui::TopBottomPanel::bottom("topic_view_bottom_panel")
        .exact_height(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Publish");
        })
}

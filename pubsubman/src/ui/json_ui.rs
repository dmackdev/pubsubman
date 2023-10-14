use serde_json::Value;

pub fn show_json_context_menu(value: &Value) -> impl FnMut(egui::Response, &String) + '_ {
    |response, pointer| {
        response
            .on_hover_cursor(egui::CursorIcon::ContextMenu)
            .context_menu(|ui| {
                show_json_context_menu_impl(ui, pointer, value);
            });
    }
}

fn show_json_context_menu_impl(ui: &mut egui::Ui, pointer: &String, value: &Value) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        ui.set_width(150.0);

        if !pointer.is_empty()
            && ui
                .add(egui::Button::new("Copy property path").frame(false))
                .clicked()
        {
            ui.output_mut(|o| o.copied_text = pointer.clone());
            ui.close_menu();
        }

        if ui
            .add(egui::Button::new("Copy contents").frame(false))
            .clicked()
        {
            if let Some(val) = value.pointer(pointer) {
                if let Ok(pretty_str) = serde_json::to_string_pretty(val) {
                    ui.output_mut(|o| o.copied_text = pretty_str);
                }
            }
            ui.close_menu();
        }
    });
}

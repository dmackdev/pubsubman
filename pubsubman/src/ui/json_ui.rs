use egui_json_tree::render::{DefaultRender, RenderContext};
use serde_json::Value;

pub fn show_json_context_menu(ui: &mut egui::Ui, context: RenderContext<'_, '_, Value>) {
    context
        .render_default(ui)
        .on_hover_cursor(egui::CursorIcon::ContextMenu)
        .context_menu(|ui| {
            let pointer = context.pointer().to_json_pointer_string();
            if !pointer.is_empty() && ui.button("Copy path").clicked() {
                ui.output_mut(|o| {
                    o.copied_text = pointer;
                });
                ui.close_menu();
            }

            if ui.button("Copy contents").clicked() {
                if let Ok(pretty_str) = serde_json::to_string_pretty(context.value()) {
                    ui.output_mut(|o| o.copied_text = pretty_str);
                }
                ui.close_menu();
            }
        });
}

use egui::{Id, Ui};

pub struct Modal<'a> {
    id: Id,
    title: &'a str,
}

impl<'a> Modal<'a> {
    pub fn new(id: impl Into<Id>, title: &'a str) -> Self {
        Self {
            id: id.into(),
            title,
        }
    }

    pub fn show<R>(&self, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui) -> R) {
        egui::Area::new(self.id)
            .interactable(true)
            .order(egui::Order::PanelResizeLine)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let screen_rect = ui.ctx().input(|i| i.screen_rect);
                ui.allocate_response(screen_rect.size(), egui::Sense::click_and_drag());

                ui.painter().rect_filled(
                    screen_rect,
                    egui::Rounding::none(),
                    egui::Color32::from_black_alpha(75),
                );
            });

        egui::Window::new(self.title)
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .show(ctx, |ui| add_contents(ui));
    }
}

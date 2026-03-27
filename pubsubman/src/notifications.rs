use egui_notify::Toasts;
use pubsubman_backend::message::BackendError;

#[derive(Default)]
pub struct Notifications {
    toasts: Toasts,
}

impl Notifications {
    pub fn show(&mut self, ctx: &egui::Context) {
        self.toasts.show(ctx);
    }

    pub fn success(&mut self, message: String) {
        self.toasts.success(message).show_progress_bar(false);
    }

    pub fn error(&mut self, error: BackendError) {
        self.toasts.error(error.to_string());
    }
}

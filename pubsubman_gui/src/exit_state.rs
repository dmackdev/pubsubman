use crate::ui::Modal;

#[derive(Default)]
pub struct ExitState {
    show_close_dialog: bool,
    can_close: bool,
    pub subscription_cleanup_state: SubscriptionCleanupState,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionCleanupState {
    #[default]
    Idle,
    Waiting,
    Complete,
}

impl ExitState {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        cleanup_subscriptions: impl FnOnce(),
    ) {
        if !self.show_close_dialog {
            return;
        }

        Modal::new("exit_modal", "Confirm Exit").show(ctx, |ui| {
            match self.subscription_cleanup_state {
                SubscriptionCleanupState::Idle => {
                    ui.label("Clean up Subscriptions?");

                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            cleanup_subscriptions();
                            self.subscription_cleanup_state = SubscriptionCleanupState::Waiting;
                        }

                        if ui.button("No").clicked() {
                            self.can_close = true;
                            frame.close();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_close_dialog = false;
                        }
                    });
                }
                SubscriptionCleanupState::Waiting => {
                    ui.label("Deleting Subscriptions...");
                }
                SubscriptionCleanupState::Complete => {
                    self.can_close = true;
                    frame.close();
                }
            }
        });
    }

    pub fn on_close_event(&mut self) -> bool {
        self.show_close_dialog = true;
        self.can_close
    }
}

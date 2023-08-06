use crate::ui::Modal;

#[derive(Default)]
pub struct ExitState {
    show_exit_dialogue: bool,
    can_exit: bool,
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
        if !self.show_exit_dialogue {
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
                            self.can_exit = true;
                            frame.close();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_exit_dialogue = false;
                        }
                    });
                }
                SubscriptionCleanupState::Waiting => {
                    ui.label("Deleting Subscriptions...");
                }
                SubscriptionCleanupState::Complete => {
                    self.can_exit = true;
                    frame.close();
                }
            }
        });
    }

    pub fn on_close_event(&mut self) -> bool {
        self.show_exit_dialogue = true;
        self.can_exit
    }
}

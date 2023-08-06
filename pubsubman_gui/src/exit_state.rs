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

const MARGIN: egui::Margin = egui::Margin {
    left: 16.0,
    right: 16.0,
    top: 12.0,
    bottom: 4.0,
};

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

        Modal::new("exit_modal", "Confirm Quit").show(ctx, |ui| {
            match self.subscription_cleanup_state {
                SubscriptionCleanupState::Idle => {
                    egui::Frame::none().inner_margin(MARGIN).show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(350.0, 150.0),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    "Pubsubman created Subscriptions in order to receive messages. Do you want to delete these Subscriptions before you quit?",
                                );

                                ui.add_space(20.0);

                                ui.horizontal(|ui| {
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            if ui.button("Delete Subscriptions").clicked() {
                                                cleanup_subscriptions();
                                                self.subscription_cleanup_state =
                                                    SubscriptionCleanupState::Waiting;
                                            }

                                            if ui.button("Skip").clicked() {
                                                self.can_exit = true;
                                                frame.close();
                                            }

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("Cancel").clicked() {
                                                        self.show_exit_dialogue = false;
                                                    }
                                                },
                                            );
                                        },
                                    );
                                });
                            },
                        );
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

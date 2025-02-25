use egui::Modal;
use pubsubman_backend::model::SubscriptionName;

#[derive(Default)]
pub struct ExitState {
    pub show_exit_dialogue: bool,
    pub can_exit: bool,
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
    left: 16,
    right: 16,
    top: 12,
    bottom: 4,
};

impl ExitState {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        sub_names: Vec<SubscriptionName>,
        cleanup_subscriptions: impl FnOnce(Vec<SubscriptionName>),
    ) {
        if !self.show_exit_dialogue {
            return;
        }

        if sub_names.is_empty() {
            self.can_exit = true;
            return;
        }

        let title = if self.subscription_cleanup_state == SubscriptionCleanupState::Waiting {
            "Deleting Subscriptions..."
        } else {
            "Confirm Quit"
        };

        Modal::new("exit_modal".into()).show(ctx, |ui| {
            egui::Frame::NONE.inner_margin(MARGIN).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.heading(title);
                    ui.add_space(20.0);
                    match self.subscription_cleanup_state {
                        SubscriptionCleanupState::Idle => {
                            self.render_dialog_contents(ui, sub_names, cleanup_subscriptions);
                        }
                        SubscriptionCleanupState::Waiting => {
                            ui.spinner();
                            ui.add_space(10.0);
                        }
                        SubscriptionCleanupState::Complete => {
                            self.can_exit = true;
                        }
                    };
                });
            });
        });
    }

    fn render_dialog_contents(
        &mut self,
        ui: &mut egui::Ui,
        sub_names: Vec<SubscriptionName>,
        cleanup_subscriptions: impl FnOnce(Vec<SubscriptionName>),
    ) {
        ui.label("Pubsubman created Subscriptions in order to receive messages.");
        ui.label("Do you want to delete these Subscriptions before you quit?");

        ui.add_space(20.0);

        ui.collapsing("Subscriptions", |ui| {
            for sub_name in sub_names.iter() {
                ui.label(&sub_name.0);
            }
        });

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                if ui.button("Delete Subscriptions").clicked() {
                    cleanup_subscriptions(sub_names);
                    self.subscription_cleanup_state = SubscriptionCleanupState::Waiting;
                }

                if ui.button("Skip").clicked() {
                    self.can_exit = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_exit_dialogue = false;
                    }
                });
            });
        });
    }
}

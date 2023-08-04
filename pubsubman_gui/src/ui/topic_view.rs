use chrono::{DateTime, Local};
use pubsubman_backend::{
    message::FrontendMessage,
    model::{PubsubMessage, SubscriptionName, TopicName},
};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

pub struct TopicViewState {
    pub selected_topic_name: TopicName,
    pub stream_messages_enabled: bool,
    pub stream_messages_cancel_token: Option<CancellationToken>,
}

impl TopicViewState {
    pub fn new(selected_topic: TopicName) -> Self {
        Self {
            selected_topic_name: selected_topic,
            stream_messages_enabled: false,
            stream_messages_cancel_token: None,
        }
    }
}

impl TopicViewState {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        front_tx: &Sender<FrontendMessage>,
        sub_name: &SubscriptionName,
        messages: &[PubsubMessage],
    ) -> egui::InnerResponse<()> {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Messages");

                ui.add_enabled_ui(!self.stream_messages_enabled, |ui| {
                    let pull_button = ui.button("Pull");
                    if pull_button.clicked() {
                        let topic_name = self.selected_topic_name.clone();
                        let sub_name = sub_name.clone();
                        let front_tx = front_tx.clone();
                        let cancel_token = CancellationToken::new();
                        let cancel_token_clone = cancel_token.clone();

                        tokio::spawn(async move {
                            front_tx
                                .send(FrontendMessage::PullMessages(
                                    topic_name,
                                    sub_name,
                                    cancel_token,
                                ))
                                .await
                                .unwrap();
                        });

                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            cancel_token_clone.cancel();
                        });
                    }
                    pull_button
                        .on_hover_text(
                            "Retrieve batch of all undelivered messages on this subscription.",
                        )
                        .on_disabled_hover_text("Disable Stream mode to Pull messages.");
                });

                let stream_mode_toggle =
                    ui.toggle_value(&mut self.stream_messages_enabled, "Stream");

                if stream_mode_toggle.changed() {
                    if self.stream_messages_enabled {
                        let topic_name = self.selected_topic_name.clone();
                        let sub_name = sub_name.clone();
                        let front_tx = front_tx.clone();
                        let cancel_token = CancellationToken::new();
                        let cancel_token_clone = cancel_token.clone();

                        tokio::spawn(async move {
                            front_tx
                                .send(FrontendMessage::PullMessages(
                                    topic_name,
                                    sub_name,
                                    cancel_token,
                                ))
                                .await
                                .unwrap();
                        });

                        self.stream_messages_cancel_token = Some(cancel_token_clone);
                    } else if let Some(cancel_token) = self.stream_messages_cancel_token.take() {
                        cancel_token.cancel();
                    }
                }

                stream_mode_toggle.on_hover_text(
                    "Continuously retrieve messages delivered to this subscription.",
                );
            });

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for message in messages.iter() {
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(0.0, egui::Color32::DARK_BLUE))
                            .show(ui, |ui| {
                                if let Some(publish_time) = message.publish_time {
                                    let local_publish_time: DateTime<Local> = publish_time.into();

                                    ui.label(format!(
                                        "{}",
                                        local_publish_time.format("%d/%m/%Y %H:%M")
                                    ));
                                }
                                ui.label(&message.data);
                            });
                    }
                });
        })
    }
}

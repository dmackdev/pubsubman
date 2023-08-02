use pubsubman_backend::{
    message::{BackendMessage, FrontendMessage},
    Backend,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::topic::Topic;

pub struct TemplateApp {
    topics: Vec<Topic>,
    front_tx: Sender<FrontendMessage>,
    back_rx: Receiver<BackendMessage>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (front_tx, front_rx) = tokio::sync::mpsc::channel(2);
        let (back_tx, back_rx) = tokio::sync::mpsc::channel(10);

        tokio::spawn(async move {
            Backend::new(back_tx, front_rx).await.init();
        });

        let front_tx_clone = front_tx.clone();
        tokio::spawn(async move {
            let _ = front_tx_clone
                .send(FrontendMessage::RefreshTopicsRequest)
                .await;
        });

        Self {
            topics: vec![],
            front_tx,
            back_rx,
        }
    }

    fn handle_backend_message(&mut self) {
        match self.back_rx.try_recv() {
            Ok(message) => match message {
                BackendMessage::TopicsUpdated(topics) => {
                    self.topics = topics.into_iter().map(|id| Topic { id }).collect();

                    let front_tx = self.front_tx.clone();
                    tokio::spawn(async move {
                        let _ = front_tx.send(FrontendMessage::RefreshTopicsRequest).await;
                    });
                }
            },
            Err(err) => println!("{:?}", err),
        }
    }

    fn render_top_panel(&self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });
    }

    fn render_topics_panel(&self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .exact_width(250.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Topics");

                for topic in self.topics.iter() {
                    topic.show(ui);
                }
            });
    }

    fn render_central_panel(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.handle_backend_message();
        self.render_top_panel(ctx, frame);
        self.render_topics_panel(ctx);
        self.render_central_panel(ctx);
    }
}

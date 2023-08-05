#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub view: ViewSettings,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ViewSettings {
    pub show_publish_message_panel: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            show_publish_message_panel: true,
        }
    }
}

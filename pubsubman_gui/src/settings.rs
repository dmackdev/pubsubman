#[derive(Default)]
pub struct Settings {
    pub view: ViewSettings,
}

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

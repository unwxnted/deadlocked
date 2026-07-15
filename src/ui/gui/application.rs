use egui::Ui;

use crate::ui::app::App;

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl App {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("deadlocked");
            ui.label("author: avitrano");
            ui.label(format!("Version: v{VERSION}"));
        });
    }
}

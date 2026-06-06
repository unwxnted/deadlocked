use egui::{DragValue, Ui};

use crate::ui::{
    app::App,
    gui::helpers::{checkbox, collapsing_open, combo_box, drag, keybind, scroll},
};

impl App {
    pub fn movement_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "movement_left", |ui| self.movement_left(ui));

            let right = &mut cols[1];
            scroll(right, "movement_right", |ui| self.movement_right(ui));
        });
    }

    fn movement_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Master Activation", |ui| {
            if checkbox(
                ui,
                "Enable Master Activation",
                &mut self.config.movement.master_enabled,
            ) {
                self.send_config();
            }

            if keybind(
                ui,
                "movement_master_hotkey",
                "Hotkey",
                &mut self.config.movement.master_hotkey,
            ) {
                self.send_config();
            }

            if combo_box(
                ui,
                "movement_master_mode",
                "Mode",
                &mut self.config.movement.master_mode,
            ) {
                self.send_config();
            }
        });

        collapsing_open(ui, "Bunnyhop", |ui| {
            if checkbox(
                ui,
                "Enable Bunnyhop",
                &mut self.config.movement.bhop.enabled,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Use Master Activation",
                &mut self.config.movement.bhop.use_master_activation,
            ) {
                self.send_config();
            }

            if !self.config.movement.bhop.use_master_activation {
                if keybind(
                    ui,
                    "movement_bhop_hotkey",
                    "Hotkey",
                    &mut self.config.movement.bhop.hotkey,
                ) {
                    self.send_config();
                }

                if combo_box(
                    ui,
                    "movement_bhop_mode",
                    "Mode",
                    &mut self.config.movement.bhop.mode,
                ) {
                    self.send_config();
                }
            } else if !self.config.movement.master_enabled {
                ui.label("Master disabled.");
            }

            ui.label("Space works best in Toggle mode.");
        });
    }

    fn movement_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Autostrafe", |ui| {
            if checkbox(
                ui,
                "Enable Autostrafe",
                &mut self.config.movement.autostrafe.enabled,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Use Master Activation",
                &mut self.config.movement.autostrafe.use_master_activation,
            ) {
                self.send_config();
            }

            if !self.config.movement.autostrafe.use_master_activation {
                if keybind(
                    ui,
                    "movement_autostrafe_hotkey",
                    "Hotkey",
                    &mut self.config.movement.autostrafe.hotkey,
                ) {
                    self.send_config();
                }

                if combo_box(
                    ui,
                    "movement_autostrafe_mode",
                    "Mode",
                    &mut self.config.movement.autostrafe.mode,
                ) {
                    self.send_config();
                }
            } else if !self.config.movement.master_enabled {
                ui.label("Master disabled.");
            }

            if drag(
                ui,
                "Smooth",
                DragValue::new(&mut self.config.movement.autostrafe.smooth)
                    .range(0.0..=1.0)
                    .speed(0.01)
                    .max_decimals(2),
            ) {
                self.send_config();
            }

            ui.label("Mouse steers. Autostrafe syncs A/D only.");
        });
    }
}

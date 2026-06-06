use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::{config::aim::KeyMode, cs2::key_codes::KeyCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UnsafeConfig {
    pub bhop: bool,
    pub bhop_hotkey: KeyCode,
    pub bhop_mode: KeyMode,
    pub no_flash: bool,
    pub max_flash_alpha: f32,
    pub fov_changer: bool,
    pub desired_fov: u32,
    pub no_smoke: bool,
    pub change_smoke_color: bool,
    pub smoke_color: Color32,
}

impl Default for UnsafeConfig {
    fn default() -> Self {
        Self {
            bhop: false,
            bhop_hotkey: KeyCode::Space,
            bhop_mode: KeyMode::Toggle,
            no_flash: false,
            max_flash_alpha: 127.0,
            fov_changer: false,
            desired_fov: 90,
            no_smoke: false,
            change_smoke_color: false,
            smoke_color: Color32::RED,
        }
    }
}

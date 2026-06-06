use serde::{Deserialize, Serialize};

use crate::{config::aim::KeyMode, cs2::key_codes::KeyCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MovementConfig {
    pub master_enabled: bool,
    pub master_hotkey: KeyCode,
    pub master_mode: KeyMode,
    pub bhop: BunnyhopConfig,
    pub autostrafe: AutostrafeConfig,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            master_enabled: false,
            master_hotkey: KeyCode::Space,
            master_mode: KeyMode::Toggle,
            bhop: BunnyhopConfig::default(),
            autostrafe: AutostrafeConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BunnyhopConfig {
    pub enabled: bool,
    pub use_master_activation: bool,
    pub hotkey: KeyCode,
    pub mode: KeyMode,
}

impl Default for BunnyhopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            use_master_activation: false,
            hotkey: KeyCode::Space,
            mode: KeyMode::Toggle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutostrafeConfig {
    pub enabled: bool,
    pub use_master_activation: bool,
    pub hotkey: KeyCode,
    pub mode: KeyMode,
    pub smooth: f32,
}

impl Default for AutostrafeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            use_master_activation: false,
            hotkey: KeyCode::Space,
            mode: KeyMode::Toggle,
            smooth: 0.35,
        }
    }
}

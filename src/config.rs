use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::IcError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetPhrase {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrtEffectsConfig {
    #[serde(default = "default_scanlines")]
    pub scanlines: bool,
    #[serde(default = "default_scanline_opacity")]
    pub scanline_opacity: f32,
    #[serde(default)]
    pub curvature: bool,
    #[serde(default = "default_curvature_amount")]
    pub curvature_amount: f32,
    #[serde(default)]
    pub grid: bool,
}

fn default_scanlines() -> bool {
    true
}
fn default_scanline_opacity() -> f32 {
    0.5
}
fn default_curvature_amount() -> f32 {
    50.0
}
fn default_volume() -> f32 {
    0.2
}

impl Default for CrtEffectsConfig {
    fn default() -> Self {
        Self {
            scanlines: default_scanlines(),
            scanline_opacity: default_scanline_opacity(),
            curvature: false,
            curvature_amount: default_curvature_amount(),
            grid: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct AppConfig {
    #[serde(default)]
    pub show_ip: bool,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default)]
    pub crt_effects: CrtEffectsConfig,
    #[serde(default = "crate::assets::default_phrases")]
    pub phrases: Vec<PresetPhrase>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            show_ip: false,
            volume: default_volume(),
            crt_effects: CrtEffectsConfig::default(),
            phrases: crate::assets::default_phrases(),
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("ic"))
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("config.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), IcError> {
        let Some(dir) = Self::config_dir() else {
            return Err(IcError::Config(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            )));
        };

        fs::create_dir_all(&dir)?;

        let path = dir.join("config.json");
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

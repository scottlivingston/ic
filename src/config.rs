use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetPhrase {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,
}

fn default_phrases() -> Vec<PresetPhrase> {
    vec![
        PresetPhrase {
            text: "I AM THE I C.".into(),
            face: None,
        },
        PresetPhrase {
            text: "Hello, Lunar traveler.".into(),
            face: None,
        },
        PresetPhrase {
            text: "Welcome to the Mall Station!".into(),
            face: None,
        },
        PresetPhrase {
            text: "Follow me to the check-in terminal and get ready for an adventure in The Mall!"
                .into(),
            face: None,
        },
        PresetPhrase {
            text: "Hello again, why are you still here.".into(),
            face: Some("angry".into()),
        },
        PresetPhrase {
            text: "You need to sign in using this terminal!".into(),
            face: None,
        },
        PresetPhrase {
            text: "Umm.. there seems to be a problem.".into(),
            face: None,
        },
        PresetPhrase {
            text: "Not to worry mister, I know another way, follow me.".into(),
            face: None,
        },
        PresetPhrase {
            text: "Oh my, What a day!".into(),
            face: None,
        },
        PresetPhrase {
            text: "I love helping people who have no clue what they are doing.".into(),
            face: None,
        },
        PresetPhrase {
            text: "Everyone comes here, Union Plaza is the best, Ha ha..".into(),
            face: None,
        },
        PresetPhrase {
            text: "Have you seen Jackie recently? I miss her.".into(),
            face: None,
        },
        PresetPhrase {
            text: "Did you know that four legs are better than two. he he he.".into(),
            face: None,
        },
        PresetPhrase {
            text: "I am happy to stay here.. yes.".into(),
            face: None,
        },
        PresetPhrase {
            text: "Good by fellow Moon traveler, Good byy!".into(),
            face: None,
        },
    ]
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
    #[serde(default = "default_phrases")]
    pub phrases: Vec<PresetPhrase>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            show_ip: false,
            volume: default_volume(),
            crt_effects: CrtEffectsConfig::default(),
            phrases: default_phrases(),
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

    pub fn save(&self) -> Result<(), std::io::Error> {
        let Some(dir) = Self::config_dir() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ));
        };

        fs::create_dir_all(&dir)?;

        let path = dir.join("config.json");
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)
    }
}

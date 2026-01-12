//! Centralized asset embedding for the IC application.
//!
//! All static assets embedded in the binary are defined here.

use bevy::prelude::Color;

/// IC face/text color: #ccffee
pub const FACE_COLOR: Color = Color::srgb(0.8, 1.0, 0.933);

// Admin web UI (served via HTTP)
pub const ADMIN_HTML: &str = include_str!("assets/admin/index.html");
pub const ADMIN_JS: &str = include_str!("assets/admin/admin.js");
pub const ADMIN_CSS: &str = include_str!("assets/admin/admin.css");
pub const ADMIN_FONT: &[u8] = include_bytes!("assets/admin/videotype.ttf");

// Face images (used by FaceLibrary and HTTP API)
pub const FACE_DEFAULT: &[u8] = include_bytes!("assets/faces/default.png");
pub const FACE_DEFAULT_TALKING: &[u8] = include_bytes!("assets/faces/default_talking.png");
pub const FACE_ANGRY: &[u8] = include_bytes!("assets/faces/angry.png");
pub const FACE_ANGRY_TALKING: &[u8] = include_bytes!("assets/faces/angry_talking.png");

// Re-export PresetPhrase for convenience
pub use crate::config::PresetPhrase;

/// Default IC phrases from the game ROUTINE
pub fn default_phrases() -> Vec<PresetPhrase> {
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

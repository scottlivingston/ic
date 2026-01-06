use bevy::prelude::*;

/// Event sent when a Say command is received
#[derive(Message, Debug, Clone)]
pub struct SayEvent {
    pub msg: String,
}

/// Event sent when an Effects command is received
#[derive(Message, Debug, Clone)]
pub struct EffectsEvent {
    pub glow: bool,
    pub glow_intensity: f32,
    pub scanlines: bool,
    pub scanline_opacity: f32,
    pub flicker: bool,
    pub flicker_amount: f32,
    pub curvature: bool,
    pub curvature_amount: f32,
    pub grid: bool,
}

/// Event sent when a Volume command is received
#[derive(Message, Debug, Clone)]
pub struct VolumeEvent {
    pub volume: f32,
}

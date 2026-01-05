use bevy::prelude::*;

/// Event sent when a Say command is received
#[derive(Message, Debug, Clone)]
pub struct SayEvent {
    pub msg: String,
}

/// Event sent when an Effects command is received
#[derive(Message, Debug, Clone)]
#[allow(dead_code)]
pub struct EffectsEvent {
    pub glow: bool,
    pub glow_intensity: f32,
    pub scanlines: bool,
    pub scanline_opacity: f32,
    pub flicker: bool,
    pub flicker_speed: f32,
    pub flicker_amount: f32,
    pub curvature: bool,
    pub curvature_amount: f32,
    pub grid: bool,
}

/// Event to signal that speech has finished and we're ready for the next message
#[derive(Message, Debug, Clone)]
pub struct ReadyEvent;

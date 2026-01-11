use bevy::prelude::*;

/// Face type for speech animation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FaceType {
    #[default]
    Default,
    Angry,
}

/// Event sent when a Say command is received
#[derive(Message, Debug, Clone)]
pub struct SayEvent {
    pub msg: String,
    pub face: FaceType,
}

/// Event sent when an Effects command is received
#[derive(Message, Debug, Clone)]
pub struct EffectsEvent {
    pub scanlines: bool,
    pub scanline_opacity: f32,
    pub curvature: bool,
    pub curvature_amount: f32,
    pub grid: bool,
}

/// Event sent when a Volume command is received
#[derive(Message, Debug, Clone)]
pub struct VolumeEvent {
    pub volume: f32,
}

/// Event sent when the IP HUD toggle changes
#[derive(Message, Debug, Clone)]
pub struct ToggleHudEvent {
    pub show: bool,
}

/// Event sent when WiFi connection succeeds
#[derive(Message, Debug, Clone)]
pub struct WifiConnectedEvent {
    pub ip: String,
}

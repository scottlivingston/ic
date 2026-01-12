use bevy::prelude::*;

/// Shared resource to track speaking state between face and audio modules
#[derive(Resource, Default)]
pub struct SpeakingState {
    pub speaking: bool,
    pub face_name: String,
}

impl SpeakingState {
    pub fn start_speaking(&mut self, face_name: String) {
        self.speaking = true;
        self.face_name = face_name;
    }

    pub fn stop_speaking(&mut self) {
        self.speaking = false;
    }
}

/// Face type for speech animation - string-based to support custom faces
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FaceType(pub String);

impl FaceType {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
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

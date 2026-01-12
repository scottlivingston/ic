//! Request and response types for the HTTP API.

use serde::{Deserialize, Serialize};

use crate::config::{CrtEffectsConfig, PresetPhrase};
use crate::wifi::WifiNetwork;

// ============================================================================
// Speak
// ============================================================================

#[derive(Deserialize)]
pub struct SpeakRequest {
    pub msg: String,
    #[serde(default)]
    pub face: Option<String>,
}

// ============================================================================
// Effects
// ============================================================================

#[derive(Deserialize)]
pub struct EffectsRequest {
    pub scanlines: bool,
    pub scanline_opacity: f32,
    pub curvature: bool,
    pub curvature_amount: f32,
    pub grid: bool,
}

// ============================================================================
// Volume
// ============================================================================

#[derive(Deserialize)]
pub struct VolumeRequest {
    pub volume: f32,
}

// ============================================================================
// Settings & Phrases
// ============================================================================

#[derive(Serialize)]
pub struct SettingsResponse {
    pub show_ip: bool,
    pub volume: f32,
    pub crt_effects: CrtEffectsConfig,
    pub phrases: Vec<PresetPhrase>,
}

#[derive(Serialize)]
pub struct SavePhrasesResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// WiFi
// ============================================================================

#[derive(Serialize)]
pub struct WifiScanResponse {
    pub networks: Vec<WifiNetwork>,
}

#[derive(Deserialize)]
pub struct WifiConnectRequest {
    pub ssid: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct WifiConnectResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct WifiForgetRequest {
    pub ssid: String,
}

#[derive(Serialize)]
pub struct WifiForgetResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct WifiStatusResponse {
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
}

// ============================================================================
// HUD
// ============================================================================

#[derive(Deserialize)]
pub struct HudToggleRequest {
    pub show: bool,
}

#[derive(Serialize)]
pub struct HudStatusResponse {
    pub show_ip: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
}

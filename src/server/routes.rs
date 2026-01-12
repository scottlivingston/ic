//! HTTP route handlers for the API.

use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use bevy::log::{error, info};

use crate::assets::{
    ADMIN_CSS, ADMIN_FONT, ADMIN_HTML, ADMIN_JS,
    FACE_ANGRY, FACE_ANGRY_TALKING, FACE_DEFAULT, FACE_DEFAULT_TALKING,
};
use crate::config::{AppConfig, CrtEffectsConfig, PresetPhrase};
use crate::wifi::{self, ConnectivityStatus};

use super::types::*;
use super::{AppState, Command};

// ============================================================================
// Static Assets
// ============================================================================

pub async fn serve_admin() -> Html<&'static str> {
    Html(ADMIN_HTML)
}

pub async fn serve_css() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css")],
        ADMIN_CSS,
    )
        .into_response()
}

pub async fn serve_js() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript")],
        ADMIN_JS,
    )
        .into_response()
}

pub async fn serve_font() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "font/ttf")],
        ADMIN_FONT,
    )
        .into_response()
}

// ============================================================================
// Core API
// ============================================================================

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn speak(State(state): State<AppState>, Json(req): Json<SpeakRequest>) -> StatusCode {
    let cmd = Command::Say {
        msg: req.msg,
        face: req.face,
    };
    if let Err(e) = state.bevy_tx.send(cmd) {
        error!("Failed to send speak command to Bevy: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

pub async fn effects(State(state): State<AppState>, Json(req): Json<EffectsRequest>) -> StatusCode {
    let cmd = Command::Effects {
        scanlines: req.scanlines,
        scanline_opacity: req.scanline_opacity,
        curvature: req.curvature,
        curvature_amount: req.curvature_amount,
        grid: req.grid,
    };
    if let Err(e) = state.bevy_tx.send(cmd) {
        error!("Failed to send effects command to Bevy: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // Save effects to config
    let mut config = AppConfig::load();
    config.crt_effects = CrtEffectsConfig {
        scanlines: req.scanlines,
        scanline_opacity: req.scanline_opacity,
        curvature: req.curvature,
        curvature_amount: req.curvature_amount,
        grid: req.grid,
    };
    if let Err(e) = config.save() {
        error!("Failed to save config: {}", e);
    }

    StatusCode::OK
}

pub async fn volume(State(state): State<AppState>, Json(req): Json<VolumeRequest>) -> StatusCode {
    let cmd = Command::Volume { volume: req.volume };
    if let Err(e) = state.bevy_tx.send(cmd) {
        error!("Failed to send volume command to Bevy: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // Save volume to config
    let mut config = AppConfig::load();
    config.volume = req.volume;
    if let Err(e) = config.save() {
        error!("Failed to save config: {}", e);
    }

    StatusCode::OK
}

// ============================================================================
// Settings & Phrases
// ============================================================================

pub async fn get_settings(State(state): State<AppState>) -> Json<SettingsResponse> {
    let config = AppConfig::load();
    let wifi = state.wifi.clone();
    let _ip_address = tokio::task::spawn_blocking(move || wifi.get_ip_address())
        .await
        .ok()
        .flatten();

    Json(SettingsResponse {
        show_ip: config.show_ip,
        volume: config.volume,
        crt_effects: config.crt_effects,
        phrases: config.phrases,
    })
}

pub async fn get_phrases() -> Json<Vec<PresetPhrase>> {
    let config = AppConfig::load();
    Json(config.phrases)
}

pub async fn save_phrases(Json(phrases): Json<Vec<PresetPhrase>>) -> Json<SavePhrasesResponse> {
    let mut config = AppConfig::load();
    config.phrases = phrases;

    match config.save() {
        Ok(()) => Json(SavePhrasesResponse {
            success: true,
            error: None,
        }),
        Err(e) => Json(SavePhrasesResponse {
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

// ============================================================================
// WiFi
// ============================================================================

pub async fn wifi_scan(State(state): State<AppState>) -> Json<WifiScanResponse> {
    let wifi = state.wifi.clone();
    // Run scan in blocking task to avoid blocking the async runtime
    let networks = tokio::task::spawn_blocking(move || wifi.scan_networks().unwrap_or_default())
        .await
        .unwrap_or_default();

    Json(WifiScanResponse { networks })
}

pub async fn wifi_connect(
    State(state): State<AppState>,
    Json(req): Json<WifiConnectRequest>,
) -> Json<WifiConnectResponse> {
    info!("Attempting to connect to WiFi: {}", req.ssid);

    let ssid = req.ssid.clone();
    let password = req.password.clone();
    let wifi = state.wifi.clone();

    let result = tokio::task::spawn_blocking(move || wifi.connect(&ssid, &password))
        .await
        .unwrap_or_else(|e| Err(wifi::WifiError::CommandFailed(e.to_string())));

    match result {
        Ok(()) => {
            // Wait a moment for IP assignment
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let wifi = state.wifi.clone();
            let ip_address = tokio::task::spawn_blocking(move || wifi.get_ip_address())
                .await
                .ok()
                .flatten();

            // Send WifiConnected command to Bevy
            if let Some(ref ip) = ip_address {
                let cmd = Command::WifiConnected { ip: ip.clone() };
                if let Err(e) = state.bevy_tx.send(cmd) {
                    error!("Failed to send WiFi connected command to Bevy: {}", e);
                }
            }

            Json(WifiConnectResponse {
                success: true,
                ip_address,
                error: None,
            })
        }
        Err(e) => Json(WifiConnectResponse {
            success: false,
            ip_address: None,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn wifi_forget(
    State(state): State<AppState>,
    Json(req): Json<WifiForgetRequest>,
) -> Json<WifiForgetResponse> {
    info!("Forgetting WiFi network: {}", req.ssid);

    let ssid = req.ssid.clone();
    let wifi = state.wifi.clone();

    let result = tokio::task::spawn_blocking(move || wifi.forget_network(&ssid))
        .await
        .unwrap_or_else(|e| Err(wifi::WifiError::CommandFailed(e.to_string())));

    match result {
        Ok(()) => Json(WifiForgetResponse {
            success: true,
            error: None,
        }),
        Err(e) => Json(WifiForgetResponse {
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn wifi_status(State(state): State<AppState>) -> Json<WifiStatusResponse> {
    let wifi = state.wifi.clone();
    let (connectivity, ssid, ip_address) = tokio::task::spawn_blocking(move || {
        let connectivity = wifi.check_connectivity();
        let ssid = wifi.get_current_ssid();
        let ip_address = wifi.get_ip_address();
        (connectivity, ssid, ip_address)
    })
    .await
    .unwrap_or((ConnectivityStatus::None, None, None));

    let connected = matches!(
        connectivity,
        ConnectivityStatus::Full | ConnectivityStatus::Limited
    );

    Json(WifiStatusResponse {
        connected,
        ssid,
        ip_address,
    })
}

// ============================================================================
// HUD
// ============================================================================

pub async fn hud_toggle(
    State(state): State<AppState>,
    Json(req): Json<HudToggleRequest>,
) -> StatusCode {
    let cmd = Command::ToggleIpHud { show: req.show };
    if let Err(e) = state.bevy_tx.send(cmd) {
        error!("Failed to send HUD toggle command to Bevy: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

pub async fn hud_status(State(state): State<AppState>) -> Json<HudStatusResponse> {
    let config = AppConfig::load();
    let wifi = state.wifi.clone();
    let ip_address = tokio::task::spawn_blocking(move || wifi.get_ip_address())
        .await
        .ok()
        .flatten();

    Json(HudStatusResponse {
        show_ip: config.show_ip,
        ip_address,
    })
}

// ============================================================================
// Faces
// ============================================================================

pub async fn list_faces(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.face_names.clone())
}

pub async fn serve_face_image(Path(name): Path<String>) -> Response {
    let bytes: Option<&'static [u8]> = match name.as_str() {
        "default.png" => Some(FACE_DEFAULT),
        "default_talking.png" => Some(FACE_DEFAULT_TALKING),
        "angry.png" => Some(FACE_ANGRY),
        "angry_talking.png" => Some(FACE_ANGRY_TALKING),
        _ => None,
    };

    match bytes {
        Some(data) => {
            (StatusCode::OK, [(header::CONTENT_TYPE, "image/png")], data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

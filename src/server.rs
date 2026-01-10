use std::sync::mpsc::Sender;
use std::sync::{mpsc, Mutex};

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bevy::log::{error, info};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::config::AppConfig;
use crate::events::{EffectsEvent, SayEvent, ToggleHudEvent, VolumeEvent, WifiConnectedEvent};
use crate::wifi::{self, ConnectivityStatus, WifiManager, WifiService};

// ============================================================================
// Plugin
// ============================================================================

/// Resource to receive commands from the web server
#[derive(Resource)]
struct ServerReceiver(Mutex<mpsc::Receiver<Command>>);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = mpsc::channel::<Command>();

        // Get WiFi service from app resources
        let wifi = app
            .world()
            .get_resource::<WifiService>()
            .expect("WifiService must be inserted before ServerPlugin")
            .0
            .clone();

        // Spawn web server in background thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(run(tx, wifi));
        });

        info!("Web server starting on http://localhost:3000");

        app.insert_resource(ServerReceiver(Mutex::new(rx)))
            .add_message::<SayEvent>()
            .add_message::<EffectsEvent>()
            .add_message::<VolumeEvent>()
            .add_message::<ToggleHudEvent>()
            .add_message::<WifiConnectedEvent>()
            .add_systems(Update, receive_server_commands);
    }
}

/// System to receive commands from the web server and dispatch as Bevy events
fn receive_server_commands(
    rx: Res<ServerReceiver>,
    mut say_events: MessageWriter<SayEvent>,
    mut effects_events: MessageWriter<EffectsEvent>,
    mut volume_events: MessageWriter<VolumeEvent>,
    mut toggle_hud_events: MessageWriter<ToggleHudEvent>,
    mut wifi_connected_events: MessageWriter<WifiConnectedEvent>,
) {
    let receiver = rx.0.lock().unwrap();
    while let Ok(cmd) = receiver.try_recv() {
        match cmd {
            Command::Say { msg } => {
                info!("Received Say command: {}", msg);
                say_events.write(SayEvent { msg });
            }
            Command::Effects {
                glow,
                glow_intensity,
                scanlines,
                scanline_opacity,
                flicker,
                flicker_amount,
                curvature,
                curvature_amount,
                grid,
            } => {
                info!("Received Effects command");
                effects_events.write(EffectsEvent {
                    glow,
                    glow_intensity,
                    scanlines,
                    scanline_opacity,
                    flicker,
                    flicker_amount,
                    curvature,
                    curvature_amount,
                    grid,
                });
            }
            Command::Volume { volume } => {
                info!("Received Volume command: {}", volume);
                volume_events.write(VolumeEvent { volume });
            }
            Command::ToggleIpHud { show } => {
                info!("Received ToggleIpHud command: {}", show);
                toggle_hud_events.write(ToggleHudEvent { show });
            }
            Command::WifiConnected { ip } => {
                info!("Received WifiConnected command: {}", ip);
                wifi_connected_events.write(WifiConnectedEvent { ip });
            }
        }
    }
}

// ============================================================================
// Web Server
// ============================================================================

/// Commands that can be sent to the Bevy app
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
enum Command {
    Say { msg: String },
    Effects {
        glow: bool,
        glow_intensity: f32,
        scanlines: bool,
        scanline_opacity: f32,
        flicker: bool,
        flicker_amount: f32,
        curvature: bool,
        curvature_amount: f32,
        grid: bool,
    },
    Volume { volume: f32 },
    ToggleIpHud { show: bool },
    WifiConnected { ip: String },
}

#[derive(Clone)]
struct AppState {
    /// Channel to send commands to Bevy
    bevy_tx: Sender<Command>,
    /// WiFi manager for network operations
    wifi: Arc<dyn WifiManager>,
}

/// Run the web server
async fn run(bevy_tx: Sender<Command>, wifi: Arc<dyn WifiManager>) {
    let state = AppState { bevy_tx, wifi };

    let api = Router::new()
        .route("/health", get(health))
        .route("/speak", post(speak))
        .route("/effects", post(effects))
        .route("/volume", post(volume))
        .route("/wifi/scan", get(wifi_scan))
        .route("/wifi/connect", post(wifi_connect))
        .route("/wifi/forget", post(wifi_forget))
        .route("/wifi/status", get(wifi_status))
        .route("/hud/ip", post(hud_toggle))
        .route("/hud/status", get(hud_status))
        .with_state(state.clone());

    let app = Router::new()
        .route("/", get(serve_admin))
        .route("/admin.css", get(serve_css))
        .route("/admin.js", get(serve_js))
        .route("/videotype.ttf", get(serve_font))
        .nest("/api", api);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to port 3000: {}. Is another instance running?", e);
            return;
        }
    };
    info!("Server running on http://localhost:3000");
    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}

// Embedded admin assets
const ADMIN_HTML: &str = include_str!("../assets/admin/index.html");
const ADMIN_CSS: &str = include_str!("../assets/admin/admin.css");
const ADMIN_JS: &str = include_str!("../assets/admin/admin.js");
const ADMIN_FONT: &[u8] = include_bytes!("../assets/admin/videotype.ttf");

async fn serve_admin() -> Html<&'static str> {
    Html(ADMIN_HTML)
}

async fn serve_css() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css")],
        ADMIN_CSS,
    )
        .into_response()
}

async fn serve_js() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript")],
        ADMIN_JS,
    )
        .into_response()
}

async fn serve_font() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "font/ttf")],
        ADMIN_FONT,
    )
        .into_response()
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

#[derive(Deserialize)]
struct SpeakRequest {
    msg: String,
}

async fn speak(State(state): State<AppState>, Json(req): Json<SpeakRequest>) -> StatusCode {
    let cmd = Command::Say { msg: req.msg };
    if let Err(e) = state.bevy_tx.send(cmd) {
        error!("Failed to send speak command to Bevy: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

#[derive(Deserialize)]
struct EffectsRequest {
    glow: bool,
    glow_intensity: f32,
    scanlines: bool,
    scanline_opacity: f32,
    flicker: bool,
    flicker_amount: f32,
    curvature: bool,
    curvature_amount: f32,
    grid: bool,
}

async fn effects(State(state): State<AppState>, Json(req): Json<EffectsRequest>) -> StatusCode {
    let cmd = Command::Effects {
        glow: req.glow,
        glow_intensity: req.glow_intensity,
        scanlines: req.scanlines,
        scanline_opacity: req.scanline_opacity,
        flicker: req.flicker,
        flicker_amount: req.flicker_amount,
        curvature: req.curvature,
        curvature_amount: req.curvature_amount,
        grid: req.grid,
    };
    if let Err(e) = state.bevy_tx.send(cmd) {
        error!("Failed to send effects command to Bevy: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

#[derive(Deserialize)]
struct VolumeRequest {
    volume: f32,
}

async fn volume(State(state): State<AppState>, Json(req): Json<VolumeRequest>) -> StatusCode {
    let cmd = Command::Volume { volume: req.volume };
    if let Err(e) = state.bevy_tx.send(cmd) {
        error!("Failed to send volume command to Bevy: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

// ============================================================================
// WiFi Endpoints
// ============================================================================

#[derive(Serialize)]
struct WifiScanResponse {
    networks: Vec<wifi::WifiNetwork>,
}

async fn wifi_scan(State(state): State<AppState>) -> Json<WifiScanResponse> {
    let wifi = state.wifi.clone();
    // Run scan in blocking task to avoid blocking the async runtime
    let networks = tokio::task::spawn_blocking(move || wifi.scan_networks().unwrap_or_default())
        .await
        .unwrap_or_default();

    Json(WifiScanResponse { networks })
}

#[derive(Deserialize)]
struct WifiConnectRequest {
    ssid: String,
    password: String,
}

#[derive(Serialize)]
struct WifiConnectResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn wifi_connect(
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

#[derive(Deserialize)]
struct WifiForgetRequest {
    ssid: String,
}

#[derive(Serialize)]
struct WifiForgetResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn wifi_forget(
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

#[derive(Serialize)]
struct WifiStatusResponse {
    connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip_address: Option<String>,
}

async fn wifi_status(State(state): State<AppState>) -> Json<WifiStatusResponse> {
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
// HUD Endpoints
// ============================================================================

#[derive(Deserialize)]
struct HudToggleRequest {
    show: bool,
}

async fn hud_toggle(
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

#[derive(Serialize)]
struct HudStatusResponse {
    show_ip: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip_address: Option<String>,
}

async fn hud_status(State(state): State<AppState>) -> Json<HudStatusResponse> {
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

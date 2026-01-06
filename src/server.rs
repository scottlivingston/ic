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
use serde::Deserialize;

use crate::events::{EffectsEvent, SayEvent, VolumeEvent};

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

        // Spawn web server in background thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(run(tx));
        });

        info!("Web server starting on http://localhost:3000");

        app.insert_resource(ServerReceiver(Mutex::new(rx)))
            .add_message::<SayEvent>()
            .add_message::<EffectsEvent>()
            .add_message::<VolumeEvent>()
            .add_systems(Update, receive_server_commands);
    }
}

/// System to receive commands from the web server and dispatch as Bevy events
fn receive_server_commands(
    rx: Res<ServerReceiver>,
    mut say_events: MessageWriter<SayEvent>,
    mut effects_events: MessageWriter<EffectsEvent>,
    mut volume_events: MessageWriter<VolumeEvent>,
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
}

#[derive(Clone)]
struct AppState {
    /// Channel to send commands to Bevy
    bevy_tx: Sender<Command>,
}

/// Run the web server
async fn run(bevy_tx: Sender<Command>) {
    let state = AppState { bevy_tx };

    let api = Router::new()
        .route("/health", get(health))
        .route("/speak", post(speak))
        .route("/effects", post(effects))
        .route("/volume", post(volume))
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

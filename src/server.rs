use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Mutex};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bevy::log::info;
use bevy::prelude::*;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::events::{EffectsEvent, ReadyEvent, SayEvent};

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
            .add_message::<ReadyEvent>()
            .add_systems(Update, receive_server_commands);
    }
}

/// System to receive commands from the web server and dispatch as Bevy events
fn receive_server_commands(
    rx: Res<ServerReceiver>,
    mut say_events: MessageWriter<SayEvent>,
    mut effects_events: MessageWriter<EffectsEvent>,
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
                    flicker_speed: 0.15, // Not used by ic-bevy shader
                    flicker_amount,
                    curvature,
                    curvature_amount,
                    grid,
                });
            }
        }
    }
}

// ============================================================================
// Web Server
// ============================================================================

/// Commands that can be sent to the Bevy app
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
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
}

#[derive(Clone)]
struct AppState {
    /// Broadcast channel for WebSocket clients
    broadcast_tx: broadcast::Sender<Command>,
    /// Channel to send commands to Bevy
    bevy_tx: Sender<Command>,
}

/// Run the web server
async fn run(bevy_tx: Sender<Command>) {
    let (broadcast_tx, _) = broadcast::channel::<Command>(16);

    let state = AppState {
        broadcast_tx,
        bevy_tx,
    };

    let api = Router::new()
        .route("/health", get(health))
        .route("/speak", post(speak))
        .route("/effects", post(effects))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    let app = Router::new()
        .route("/", get(serve_admin))
        .route("/admin", get(serve_admin))
        .route("/admin.css", get(serve_css))
        .route("/admin.js", get(serve_js))
        .route("/videotype.ttf", get(serve_font))
        .nest("/api", api);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
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
    // Send to both broadcast (for WebSocket clients) and Bevy
    let _ = state.broadcast_tx.send(cmd.clone());
    let _ = state.bevy_tx.send(cmd);
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
    // Send to both broadcast (for WebSocket clients) and Bevy
    let _ = state.broadcast_tx.send(cmd.clone());
    let _ = state.bevy_tx.send(cmd);
    StatusCode::OK
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let mut rx = state.broadcast_tx.subscribe();
    let (mut sender, mut receiver) = socket.split();
    let mut buffer: VecDeque<Command> = VecDeque::new();
    let mut waiting_for_ready = false;

    loop {
        tokio::select! {
            // Receive from broadcast channel
            Ok(cmd) = rx.recv() => {
                match &cmd {
                    Command::Say { .. } => {
                        // Queue say commands
                        buffer.push_back(cmd);
                        if !waiting_for_ready {
                            if let Some(msg) = buffer.pop_front() {
                                let json = serde_json::to_string(&msg).unwrap();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                                waiting_for_ready = true;
                            }
                        }
                    }
                    Command::Effects { .. } => {
                        // Send effects immediately (don't queue)
                        let json = serde_json::to_string(&cmd).unwrap();
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            // Receive from client
            Some(Ok(msg)) = receiver.next() => {
                if let Message::Text(text) = msg {
                    // Client sent "ready", send next message
                    if text == "ready" {
                        waiting_for_ready = false;
                        if let Some(msg) = buffer.pop_front() {
                            let json = serde_json::to_string(&msg).unwrap();
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                            waiting_for_ready = true;
                        }
                    }
                }
            }
            else => break,
        }
    }
}

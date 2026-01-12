//! HTTP server plugin for the IC application.
//!
//! Provides a web interface for controlling the IC face via REST API.

mod routes;
mod types;

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, mpsc};

use axum::Router;
use axum::routing::{get, post};
use bevy::log::{error, info};
use bevy::prelude::*;
use serde::Deserialize;

use crate::events::{
    EffectsEvent, FaceType, SayEvent, ToggleHudEvent, VolumeEvent, WifiConnectedEvent,
};
use crate::face_library::FaceLibrary;
use crate::wifi::{WifiManager, WifiService};

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

        // Get available face names from FaceLibrary
        let face_names: Vec<String> = app
            .world()
            .get_resource::<FaceLibrary>()
            .expect("FaceLibrary must be inserted before ServerPlugin")
            .list_faces()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        // Spawn web server in background thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(run_server(tx, wifi, face_names));
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
            Command::Say { msg, face } => {
                info!("Received Say command: {} (face: {:?})", msg, face);
                let face_type = match face {
                    Some(name) => FaceType::new(name),
                    None => FaceType::default(),
                };
                say_events.write(SayEvent {
                    msg,
                    face: face_type,
                });
            }
            Command::Effects {
                scanlines,
                scanline_opacity,
                curvature,
                curvature_amount,
                grid,
            } => {
                info!("Received Effects command");
                effects_events.write(EffectsEvent {
                    scanlines,
                    scanline_opacity,
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
pub(crate) enum Command {
    Say {
        msg: String,
        face: Option<String>,
    },
    Effects {
        scanlines: bool,
        scanline_opacity: f32,
        curvature: bool,
        curvature_amount: f32,
        grid: bool,
    },
    Volume {
        volume: f32,
    },
    ToggleIpHud {
        show: bool,
    },
    WifiConnected {
        ip: String,
    },
}

#[derive(Clone)]
pub(crate) struct AppState {
    /// Channel to send commands to Bevy
    pub bevy_tx: Sender<Command>,
    /// WiFi manager for network operations
    pub wifi: Arc<dyn WifiManager>,
    /// Available face names
    pub face_names: Vec<String>,
}

/// Run the web server
async fn run_server(bevy_tx: Sender<Command>, wifi: Arc<dyn WifiManager>, face_names: Vec<String>) {
    let state = AppState {
        bevy_tx,
        wifi,
        face_names,
    };

    let api = Router::new()
        .route("/health", get(routes::health))
        .route("/speak", post(routes::speak))
        .route("/effects", post(routes::effects))
        .route("/volume", post(routes::volume))
        .route("/settings", get(routes::get_settings))
        .route("/phrases", get(routes::get_phrases))
        .route("/phrases", post(routes::save_phrases))
        .route("/wifi/scan", get(routes::wifi_scan))
        .route("/wifi/connect", post(routes::wifi_connect))
        .route("/wifi/forget", post(routes::wifi_forget))
        .route("/wifi/status", get(routes::wifi_status))
        .route("/hud/ip", post(routes::hud_toggle))
        .route("/hud/status", get(routes::hud_status))
        .route("/faces", get(routes::list_faces))
        .route("/faces/{name}", get(routes::serve_face_image))
        .with_state(state.clone());

    let app = Router::new()
        .route("/", get(routes::serve_admin))
        .route("/admin.js", get(routes::serve_js))
        .route("/admin.css", get(routes::serve_css))
        .route("/videotype.ttf", get(routes::serve_font))
        .nest("/api", api);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            error!(
                "Failed to bind to port 3000: {}. Is another instance running?",
                e
            );
            return;
        }
    };
    info!("Server running on http://localhost:3000");
    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}

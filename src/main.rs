// Feature gate conventions:
// - Module-level gating (#[cfg(feature)] on mod): Optional features that can be excluded
// - Inline gating (#[cfg(feature)] inside module): Platform-specific implementations within required modules

mod app_state;
mod assets;
mod audio;
mod config;
mod crt;
#[cfg(feature = "diagnostics")]
mod diagnostics; // Optional FPS overlay
mod error;
mod events;
mod face_library;
mod hud;
mod onboarding;
mod sam;
mod server;
mod simple_face;
mod wifi; // Contains inline #[cfg(feature = "pi")] for mock vs real implementations

use bevy::prelude::*;
#[cfg(feature = "pi")]
use bevy::window::{CursorOptions, Window};
use bevy::window::{MonitorSelection, PresentMode, WindowMode};

use app_state::AppMode;
use audio::AudioPlugin;
use config::AppConfig;
#[cfg(feature = "diagnostics")]
use diagnostics::DiagnosticsPlugin;
use face_library::FaceLibrary;
use hud::HudPlugin;
use onboarding::OnboardingPlugin;
use server::ServerPlugin;
use wifi::WifiService;

use crt::{CrtPlugin, CrtSettings};
use simple_face::SimpleFacePlugin;

#[cfg(debug_assertions)]
fn window_drag(mouse: Res<ButtonInput<MouseButton>>, mut windows: Query<&mut Window>) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(mut window) = windows.single_mut() {
            window.start_drag_move();
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        CrtSettings::default(), // CRT post-process effects
    ));
}

#[cfg(feature = "pi")]
fn hide_cursor(mut cursor_query: Query<&mut CursorOptions, With<Window>>) {
    for mut cursor in cursor_query.iter_mut() {
        cursor.visible = false;
    }
}

fn main() {
    let mut app = App::new();

    // Load config from disk (or use defaults)
    let config = AppConfig::load();
    app.insert_resource(config);

    // Load face library (embedded defaults + user custom faces)
    let face_library = FaceLibrary::load();
    app.insert_resource(face_library);

    // Insert WiFi service: mock without pi feature, real with pi feature
    #[cfg(not(feature = "pi"))]
    app.insert_resource(WifiService::new(wifi::MockWifi::new()));

    #[cfg(feature = "pi")]
    app.insert_resource(WifiService::new(wifi::NetworkManagerWifi));

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "IC".into(),
                    resolution: (640, 480).into(),
                    decorations: false,
                    present_mode: PresentMode::Mailbox,
                    mode: if cfg!(feature = "pi") {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                    } else {
                        WindowMode::Windowed
                    },
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .init_state::<AppMode>()
    .add_systems(Startup, spawn_camera)
    .add_plugins(ServerPlugin)
    .add_plugins(SimpleFacePlugin)
    .add_plugins(CrtPlugin)
    .add_plugins(OnboardingPlugin)
    .add_plugins(HudPlugin)
    .add_plugins(AudioPlugin);

    #[cfg(feature = "diagnostics")]
    app.add_plugins(DiagnosticsPlugin);

    #[cfg(debug_assertions)]
    app.add_systems(Update, window_drag);

    #[cfg(feature = "pi")]
    app.add_systems(Startup, hide_cursor);

    app.run();
}

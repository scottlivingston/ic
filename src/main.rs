#[cfg(feature = "generate-assets")]
mod asset_gen;

mod app_state;
mod audio;
mod config;
mod crt;
mod diagnostics;
mod events;
mod face;
mod hud;
mod onboarding;
mod pixel_face;
mod sam;
mod server;
mod simple_face;
mod sprite_face;
mod wifi;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
#[cfg(not(debug_assertions))]
use bevy::window::{CursorOptions, Window};

use app_state::AppMode;
use audio::AudioPlugin;
use diagnostics::DiagnosticsPlugin;
use hud::HudPlugin;
use onboarding::OnboardingPlugin;
use server::ServerPlugin;
use wifi::WifiService;

// Face rendering options (uncomment ONE):
// Option 1: Simple sprite face - individual sprites per lit pixel (best for Pi Zero 2)
use simple_face::SimpleFacePlugin;
// Option 2: Pixel face with real-time shader effects
// use pixel_face::{PixelFacePlugin, PixelFaceSettings};
// Option 3: Sprite face with pre-rendered assets
// use sprite_face::{SpriteFacePlugin, CrtSimpleSettings};
// Option 4: Smooth sprite face with CRT shader
// use face::FacePlugin;
// CRT post-process effects (works with any face plugin)
use crt::{CrtPlugin, CrtSettings};

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

#[cfg(not(debug_assertions))]
fn hide_cursor(mut cursor_query: Query<&mut CursorOptions, With<Window>>) {
    for mut cursor in cursor_query.iter_mut() {
        cursor.visible = false;
    }
}

// Asset generation mode: run with `cargo run --features generate-assets`
#[cfg(feature = "generate-assets")]
fn main() {
    asset_gen::generate_all_assets();
}

// Normal runtime mode
#[cfg(not(feature = "generate-assets"))]
fn main() {
    let mut app = App::new();

    // Insert WiFi service: mock in debug, real in release
    #[cfg(debug_assertions)]
    app.insert_resource(WifiService::new(wifi::MockWifi::new()));

    #[cfg(not(debug_assertions))]
    app.insert_resource(WifiService::new(wifi::NetworkManagerWifi));

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "IC".into(),
                    resolution: (640, 480).into(),
                    decorations: false,
                    present_mode: PresentMode::Mailbox,
                    mode: if cfg!(debug_assertions) {
                        WindowMode::Windowed
                    } else {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
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
    // Face plugin options (uncomment ONE, match camera settings above):
    // Option 1: Simple face - individual sprites (best for Pi Zero 2)
    .add_plugins(SimpleFacePlugin)
    // Option 2: Pixel face - real-time shader effects
    // .add_plugins(PixelFacePlugin)
    // Option 3: Sprite face - pre-rendered assets
    // .add_plugins(SpriteFacePlugin)
    // Option 4: Smooth face - CRT shader
    // .add_plugins(FacePlugin)
    // CRT post-process effects (works with any face plugin, keep glow off for Pi Zero 2)
    .add_plugins(CrtPlugin)
    .add_plugins(OnboardingPlugin)
    .add_plugins(HudPlugin)
    .add_plugins(DiagnosticsPlugin)
    .add_plugins(AudioPlugin);

    #[cfg(debug_assertions)]
    app.add_systems(Update, window_drag);

    #[cfg(not(debug_assertions))]
    app.add_systems(Startup, hide_cursor);

    app.run();
}

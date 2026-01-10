mod app_state;
mod audio;
mod config;
mod crt;
mod events;
mod face;
mod hud;
mod onboarding;
mod sam;
mod server;
mod wifi;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};

use app_state::AppMode;
use audio::AudioPlugin;
use crt::{CrtPlugin, CrtSettings};
use face::FacePlugin;
use hud::HudPlugin;
use onboarding::OnboardingPlugin;
use server::ServerPlugin;
use wifi::WifiService;

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
        CrtSettings::default(),
    ));
}

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
                    present_mode: PresentMode::AutoVsync,
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
    .add_plugins(CrtPlugin)
    .add_plugins(OnboardingPlugin)
    .add_plugins(FacePlugin)
    .add_plugins(HudPlugin)
    .add_plugins(AudioPlugin);

    #[cfg(debug_assertions)]
    app.add_systems(Update, window_drag);

    app.run();
}

mod audio;
mod crt;
mod events;
mod face;
mod sam;
mod server;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};

use audio::AudioPlugin;
use crt::CrtPlugin;
use face::FacePlugin;
use server::ServerPlugin;

#[cfg(debug_assertions)]
fn window_drag(mouse: Res<ButtonInput<MouseButton>>, mut windows: Query<&mut Window>) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(mut window) = windows.single_mut() {
            window.start_drag_move();
        }
    }
}

fn main() {
    let mut app = App::new();

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
    .add_plugins(ServerPlugin)
    .add_plugins(CrtPlugin)
    .add_plugins(FacePlugin)
    .add_plugins(AudioPlugin);

    #[cfg(debug_assertions)]
    app.add_systems(Update, window_drag);

    app.run();
}

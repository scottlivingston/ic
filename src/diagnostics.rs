use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::app_state::AppMode;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticsState>()
            .init_resource::<SystemTimings>()
            .add_systems(OnEnter(AppMode::Normal), spawn_diagnostics_display)
            .add_systems(OnExit(AppMode::Normal), despawn_diagnostics_display)
            .add_systems(
                Update,
                update_diagnostics_display.run_if(in_state(AppMode::Normal)),
            );
    }
}

/// Timing data from various systems for profiling
#[derive(Resource, Default)]
pub struct SystemTimings {
    pub face_update_ms: f32,
    pub glow_update_ms: f32,
}

#[derive(Component)]
struct DiagnosticsDisplay;

#[derive(Resource)]
struct DiagnosticsState {
    fps_samples: Vec<f32>,
    sample_index: usize,
}

impl Default for DiagnosticsState {
    fn default() -> Self {
        Self {
            fps_samples: vec![0.0; 30], // 30-frame rolling average
            sample_index: 0,
        }
    }
}

// Face color: #ccffee
const TEXT_COLOR: Color = Color::srgb(0.8, 1.0, 0.933);

fn spawn_diagnostics_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("embedded://ic/assets/videotype.ttf");

    // Note: Text2d won't be visible with pixel_face (fullscreen post-process covers it)
    // For pixel_face testing, judge performance by smoothness rather than FPS numbers
    commands.spawn((
        DiagnosticsDisplay,
        Text2d::new("FPS: --"),
        TextFont {
            font,
            font_size: 20.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Anchor::TOP_LEFT,
        Transform::from_xyz(-300.0, 220.0, 10.0),
    ));
}

fn despawn_diagnostics_display(mut commands: Commands, query: Query<Entity, With<DiagnosticsDisplay>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_diagnostics_display(
    time: Res<Time>,
    timings: Res<SystemTimings>,
    mut state: ResMut<DiagnosticsState>,
    mut query: Query<&mut Text2d, With<DiagnosticsDisplay>>,
) {
    let delta = time.delta_secs();
    let fps = if delta > 0.0 { 1.0 / delta } else { 0.0 };
    let frame_ms = delta * 1000.0;

    // Update rolling average
    let idx = state.sample_index;
    let len = state.fps_samples.len();
    state.fps_samples[idx] = fps;
    state.sample_index = (idx + 1) % len;
    let avg_fps: f32 = state.fps_samples.iter().sum::<f32>() / state.fps_samples.len() as f32;

    // Estimate render time = total - measured CPU systems
    let cpu_ms = timings.face_update_ms + timings.glow_update_ms;
    let render_ms = (frame_ms - cpu_ms).max(0.0);

    for mut text in query.iter_mut() {
        text.0 = format!(
            "FPS: {:.0} ({:.0})\n{:.1}ms\nFace:{:.1} Glow:{:.1}\nRender:{:.1}",
            fps, avg_fps, frame_ms, timings.face_update_ms, timings.glow_update_ms, render_ms
        );
    }
}

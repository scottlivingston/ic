use bevy::prelude::*;

use crate::crt::CrtSettings;

pub struct FacePlugin;

impl Plugin for FacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpeakingState>()
            .add_systems(Startup, spawn_face)
            .add_systems(Update, animate_mouth);
    }
}

// Face dimensions (scaled 1.25x from original)
pub const EYE_WIDTH: f32 = 50.0;
pub const EYE_HEIGHT: f32 = 100.0;
pub const MOUTH_WIDTH: f32 = 75.0;
pub const MOUTH_HEIGHT: f32 = 15.0;
const MOUTH_HEIGHT_SPEAKING: f32 = 44.0;
pub const EYE_GAP: f32 = 300.0; // increased for more horizontal space between eyes and mouth
// Golden ratio positioning: shorter segment from top
// φ = 1.618..., shorter ratio = 1 - 1/φ ≈ 0.382
// For 480px height: 0.382 * 480 ≈ 183px from top
// In Bevy coords (center=0): 240 - 183 ≈ 57
pub const FACE_Y_OFFSET: f32 = 57.0;
// Base Y position for mouth (bottom of mouth aligns with bottom of eyes)
pub const MOUTH_BASE_Y: f32 = FACE_Y_OFFSET - EYE_HEIGHT / 2.0 + MOUTH_HEIGHT / 2.0;

// Face color: #ccffee
const FACE_COLOR: Color = Color::srgb(0.8, 1.0, 0.933);

#[derive(Component)]
pub struct Eye;

#[derive(Component)]
pub struct Mouth;

#[derive(Resource, Default)]
pub struct SpeakingState {
    pub speaking: bool,
    animation_timer: Timer,
    animation_phase: f32,
}

impl SpeakingState {
    pub fn start_speaking(&mut self) {
        self.speaking = true;
        self.animation_timer = Timer::from_seconds(1.0 / 15.0, TimerMode::Repeating);
        self.animation_phase = 0.0;
    }

    pub fn stop_speaking(&mut self) {
        self.speaking = false;
        self.animation_phase = 0.0;
    }
}

fn spawn_face(mut commands: Commands) {
    // Camera with CRT settings and black background
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        CrtSettings::default(),
    ));

    // Left eye
    commands.spawn((
        Eye,
        Sprite {
            color: FACE_COLOR,
            custom_size: Some(Vec2::new(EYE_WIDTH, EYE_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(-EYE_GAP / 2.0 - MOUTH_WIDTH / 2.0, FACE_Y_OFFSET, 0.0),
    ));

    // Right eye
    commands.spawn((
        Eye,
        Sprite {
            color: FACE_COLOR,
            custom_size: Some(Vec2::new(EYE_WIDTH, EYE_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(EYE_GAP / 2.0 + MOUTH_WIDTH / 2.0, FACE_Y_OFFSET, 0.0),
    ));

    // Mouth (centered, slightly below eyes)
    commands.spawn((
        Mouth,
        Sprite {
            color: FACE_COLOR,
            custom_size: Some(Vec2::new(MOUTH_WIDTH, MOUTH_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, MOUTH_BASE_Y, 0.0),
    ));
}

fn animate_mouth(
    time: Res<Time>,
    mut speaking_state: ResMut<SpeakingState>,
    mut mouth_query: Query<(&mut Sprite, &mut Transform), With<Mouth>>,
) {
    let Ok((mut sprite, mut transform)) = mouth_query.single_mut() else {
        return;
    };

    if speaking_state.speaking {
        speaking_state.animation_timer.tick(time.delta());

        if speaking_state.animation_timer.just_finished() {
            // Toggle between 0 and 1
            speaking_state.animation_phase = if speaking_state.animation_phase == 0.0 { 1.0 } else { 0.0 };
        }

        // Toggle between closed and open
        let height = if speaking_state.animation_phase == 0.0 {
            MOUTH_HEIGHT
        } else {
            MOUTH_HEIGHT_SPEAKING
        };

        // Offset Y to keep bottom edge fixed (sprites scale from center)
        let y_offset = (height - MOUTH_HEIGHT) / 2.0;
        transform.translation.y = MOUTH_BASE_Y + y_offset;
        sprite.custom_size = Some(Vec2::new(MOUTH_WIDTH, height));
    } else {
        // Reset to default height and position when not speaking
        sprite.custom_size = Some(Vec2::new(MOUTH_WIDTH, MOUTH_HEIGHT));
        transform.translation.y = MOUTH_BASE_Y;
    }
}

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::app_state::AppMode;
use crate::assets::FACE_COLOR;
use crate::config::AppConfig;
use crate::events::ToggleHudEvent;
use crate::wifi::WifiService;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // Font is registered by OnboardingPlugin

        app.init_resource::<HudState>()
            .add_systems(OnEnter(AppMode::Normal), (load_hud_config, spawn_ip_hud))
            .add_systems(OnExit(AppMode::Normal), despawn_ip_hud)
            .add_systems(
                Update,
                (handle_toggle_hud, update_ip_display).run_if(in_state(AppMode::Normal)),
            );
    }
}

#[derive(Resource)]
pub struct HudState {
    pub show_ip: bool,
    pub ip_address: Option<String>,
    pub from_onboarding: bool,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            show_ip: false,
            ip_address: None,
            from_onboarding: false,
        }
    }
}

#[derive(Component)]
struct IpHud;

fn load_hud_config(
    wifi: Res<WifiService>,
    mut hud_state: ResMut<HudState>,
    previous_state: Res<State<AppMode>>,
) {
    let config = AppConfig::load();

    // Auto-show IP if coming from onboarding
    let from_onboarding = matches!(previous_state.get(), AppMode::Onboarding);
    hud_state.show_ip = config.show_ip || from_onboarding;
    hud_state.from_onboarding = from_onboarding;
    hud_state.ip_address = wifi.0.get_ip_address();
}

fn spawn_ip_hud(mut commands: Commands, asset_server: Res<AssetServer>, hud_state: Res<HudState>) {
    let font = asset_server.load("embedded://ic/assets/admin/videotype.ttf");

    let text_font = TextFont {
        font,
        font_size: 20.0,
        ..default()
    };

    let ip_text = if let Some(ip) = &hud_state.ip_address {
        format!("IP: {}", ip)
    } else {
        "IP: ---.---.---.---".to_string()
    };

    // Position in bottom-left corner
    // Screen is 640x480, center is (0,0)
    // Bottom-left would be around (-280, -200)
    commands.spawn((
        IpHud,
        Text2d::new(ip_text),
        text_font,
        TextColor(FACE_COLOR),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(-290.0, -200.0, 1.0),
        Visibility::Visible,
    ));

    // Update visibility based on state
    // This will be handled by update_ip_display system
}

fn despawn_ip_hud(mut commands: Commands, query: Query<Entity, With<IpHud>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_toggle_hud(mut events: MessageReader<ToggleHudEvent>, mut hud_state: ResMut<HudState>) {
    for event in events.read() {
        hud_state.show_ip = event.show;

        // Save to config
        let mut config = AppConfig::load();
        config.show_ip = event.show;
        if let Err(e) = config.save() {
            error!("Failed to save config: {}", e);
        }
    }
}

fn update_ip_display(
    hud_state: Res<HudState>,
    mut query: Query<(&mut Text2d, &mut Visibility), With<IpHud>>,
) {
    for (mut text, mut visibility) in query.iter_mut() {
        // Update visibility
        *visibility = if hud_state.show_ip {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // Update IP text if changed
        let ip_text = if let Some(ip) = &hud_state.ip_address {
            format!("IP: {}", ip)
        } else {
            "IP: ---.---.---.---".to_string()
        };

        if text.0 != ip_text {
            text.0 = ip_text;
        }
    }
}

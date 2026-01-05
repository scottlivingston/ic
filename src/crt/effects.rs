use bevy::log::info;
use bevy::prelude::*;

use crate::events::EffectsEvent;

/// Resource holding the current CRT effect settings
#[derive(Resource, Clone, Copy)]
pub struct CrtEffects {
    pub glow_enabled: bool,
    pub glow_intensity: f32,
    pub scanlines_enabled: bool,
    pub scanline_opacity: f32,
    pub flicker_enabled: bool,
    pub flicker_amount: f32,
    pub curvature_enabled: bool,
    pub curvature_amount: f32,
    pub grid_enabled: bool,
}

impl Default for CrtEffects {
    fn default() -> Self {
        // 50% defaults to match admin menu
        Self {
            glow_enabled: true,
            glow_intensity: 0.5,
            scanlines_enabled: true,
            scanline_opacity: 0.5,
            flicker_enabled: true,
            flicker_amount: 0.15,
            curvature_enabled: true,
            curvature_amount: 50.0,
            grid_enabled: false,
        }
    }
}

/// System to update CRT effects from events
pub fn update_effects_from_events(
    mut effects_events: MessageReader<EffectsEvent>,
    mut crt_effects: ResMut<CrtEffects>,
) {
    for event in effects_events.read() {
        crt_effects.glow_enabled = event.glow;
        crt_effects.glow_intensity = event.glow_intensity;
        crt_effects.scanlines_enabled = event.scanlines;
        crt_effects.scanline_opacity = event.scanline_opacity;
        crt_effects.flicker_enabled = event.flicker;
        crt_effects.flicker_amount = event.flicker_amount;
        crt_effects.curvature_enabled = event.curvature;
        crt_effects.curvature_amount = event.curvature_amount;
        crt_effects.grid_enabled = event.grid;

        info!("CRT effects updated: glow={}, scanlines={}, flicker={}, curvature={}, grid={}",
            event.glow, event.scanlines, event.flicker, event.curvature, event.grid);
    }
}

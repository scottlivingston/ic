use bevy::log::info;
use bevy::prelude::*;

use crate::events::EffectsEvent;

/// Resource holding the current CRT effect settings
#[derive(Resource, Clone, Copy)]
pub struct CrtEffects {
    pub scanlines_enabled: bool,
    pub scanline_opacity: f32,
    pub curvature_enabled: bool,
    pub curvature_amount: f32,
    pub grid_enabled: bool,
}

impl Default for CrtEffects {
    fn default() -> Self {
        Self {
            scanlines_enabled: true,
            scanline_opacity: 0.5,
            curvature_enabled: false,
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
        crt_effects.scanlines_enabled = event.scanlines;
        crt_effects.scanline_opacity = event.scanline_opacity;
        crt_effects.curvature_enabled = event.curvature;
        crt_effects.curvature_amount = event.curvature_amount;
        crt_effects.grid_enabled = event.grid;

        info!("CRT effects updated: scanlines={}, curvature={}, grid={}",
            event.scanlines, event.curvature, event.grid);
    }
}

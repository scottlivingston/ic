// Minimal CRT Shader for Pre-rendered Sprites
// Only applies barrel distortion and vignette - glow/scanlines are pre-baked in textures

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct CrtSimpleSettings {
    curvature_enabled: u32,
    curvature_amount: f32,
    flicker_enabled: u32,
    flicker_amount: f32,
    time: f32,
    _padding: f32,
    screen_size: vec2<f32>,
}

@group(0) @binding(2) var<uniform> settings: CrtSimpleSettings;

fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let dist_sq = dot(centered, centered);
    return uv + centered * dist_sq * amount * 0.5;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    var edge_fade = 1.0;

    // Barrel distortion (curvature)
    if settings.curvature_enabled != 0u {
        uv = barrel_distort(uv, settings.curvature_amount * 0.01);

        // Edge fade for out-of-bounds areas
        let edge_width = 0.02;
        let fade_x = smoothstep(0.0, edge_width, uv.x) * smoothstep(0.0, edge_width, 1.0 - uv.x);
        let fade_y = smoothstep(0.0, edge_width, uv.y) * smoothstep(0.0, edge_width, 1.0 - uv.y);
        edge_fade = fade_x * fade_y;

        // Black outside valid UV range
        if uv.x < -0.01 || uv.x > 1.01 || uv.y < -0.01 || uv.y > 1.01 {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }

    // Sample the pre-rendered scene (sprites already composited by Bevy)
    var color = textureSample(screen_texture, texture_sampler, uv).rgb;

    // Flicker effect - brightness variation over time
    if settings.flicker_enabled != 0u {
        let flicker = 1.0 - settings.flicker_amount * 0.30 * (
            sin(settings.time * 60.0) * 0.5 +
            sin(settings.time * 113.0) * 0.3 +
            sin(settings.time * 37.0) * 0.2
        );
        color = color * flicker;
    }

    // Vignette - dim pixels at edges
    if settings.curvature_enabled != 0u {
        let centered = uv - 0.5;
        let dist = length(centered) * 2.0;
        // Start at 75% toward edge, full at 105%
        let vignette = smoothstep(0.75, 1.05, dist);
        // Dim by up to 75% at edges
        color = color * (1.0 - vignette * 0.75);
    }

    color = color * edge_fade;
    return vec4<f32>(color, 1.0);
}

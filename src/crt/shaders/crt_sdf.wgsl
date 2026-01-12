// CRT Post-Processing Shader
// Implements: scanlines, curvature (barrel distortion), grid

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct CrtSettings {
    scanlines_enabled: u32,
    scanline_opacity: f32,
    curvature_enabled: u32,
    curvature_amount: f32,
    grid_enabled: u32,
    time: f32,
    screen_size: vec2<f32>,
}

@group(0) @binding(2) var<uniform> settings: CrtSettings;

// Barrel distortion for CRT curvature effect
fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let dist_sq = dot(centered, centered);
    let distorted = uv + centered * dist_sq * amount * 0.5;
    return distorted;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Edge fade factor for antialiasing (1.0 = fully visible, 0.0 = faded to black)
    var edge_fade = 1.0;

    // Apply barrel distortion (curvature) first
    if settings.curvature_enabled != 0u {
        uv = barrel_distort(uv, settings.curvature_amount * 0.01);

        // Smooth edge falloff instead of hard cutoff for antialiasing
        let edge_width = 0.02;
        let fade_x = smoothstep(0.0, edge_width, uv.x) * smoothstep(0.0, edge_width, 1.0 - uv.x);
        let fade_y = smoothstep(0.0, edge_width, uv.y) * smoothstep(0.0, edge_width, 1.0 - uv.y);
        edge_fade = fade_x * fade_y;

        // Still clip if completely outside bounds
        if uv.x < -0.01 || uv.x > 1.01 || uv.y < -0.01 || uv.y > 1.01 {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }

    // Sample the screen texture
    var color = textureSample(screen_texture, texture_sampler, uv).rgb;

    // Apply scanlines
    if settings.scanlines_enabled != 0u {
        let screen_y = uv.y * settings.screen_size.y;
        // Create horizontal scanlines every 4 pixels
        let scanline = step(0.5, fract(screen_y * 0.25));
        // Make scanlines more visible by using higher opacity multiplier
        color = mix(color, color * 0.3, scanline * settings.scanline_opacity * 2.0);
    }

    // Apply grid overlay
    if settings.grid_enabled != 0u {
        let screen_pos = uv * settings.screen_size;
        let grid_size = 40.0;
        let grid_line = max(
            step(0.98, fract(screen_pos.x / grid_size)),
            step(0.98, fract(screen_pos.y / grid_size))
        );

        // Grid color: rgba(0, 255, 170, 0.1) = #00ffaa
        let grid_color = vec3<f32>(0.0, 1.0, 0.667);
        color = mix(color, grid_color, grid_line * 0.1);
    }

    // Apply edge fade for smooth antialiased edges on curvature
    color = color * edge_fade;

    return vec4<f32>(color, 1.0);
}

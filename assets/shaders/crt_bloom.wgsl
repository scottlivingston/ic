// CRT Post-Processing Shader (Bloom-based glow)
// Implements: glow (via brightness sampling), scanlines, flicker, curvature (barrel distortion), grid

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct CrtSettings {
    glow_enabled: u32,
    glow_intensity: f32,
    scanlines_enabled: u32,
    scanline_opacity: f32,
    flicker_enabled: u32,
    flicker_amount: f32,
    curvature_enabled: u32,
    curvature_amount: f32,
    grid_enabled: u32,
    time: f32,
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(2) var<uniform> settings: CrtSettings;

// Barrel distortion for CRT curvature effect
fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let dist_sq = dot(centered, centered);
    let distorted = uv + centered * dist_sq * amount * 0.5;
    return distorted;
}

// Simple pseudo-random for flicker
fn random(seed: f32) -> f32 {
    return fract(sin(seed * 12.9898) * 43758.5453);
}

// Get luminance of a color
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3(0.299, 0.587, 0.114));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Apply barrel distortion (curvature) first
    if settings.curvature_enabled != 0u {
        uv = barrel_distort(uv, settings.curvature_amount * 0.01);

        // Check if we're outside the screen bounds after distortion
        if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }

    // Sample the screen texture
    var color = textureSample(screen_texture, texture_sampler, uv).rgb;

    // Apply glow using golden angle spiral sampling
    if settings.glow_enabled != 0u {
        let intensity = settings.glow_intensity;
        let pixel_size = 1.0 / settings.screen_size;

        var glow_accum = 0.0;
        let radius = 30.0;  // Max radius in pixels
        let num_samples = 256;  // Total samples in spiral
        let golden_angle = 2.39996323;  // π * (3 - √5) radians

        // Sample in golden angle spiral - optimal distribution
        for (var i = 1; i <= num_samples; i++) {
            // Distance increases with sqrt for even area distribution
            let r = radius * sqrt(f32(i) / f32(num_samples));
            let angle = f32(i) * golden_angle;

            let offset = vec2(cos(angle), sin(angle)) * r * pixel_size;
            let sample_color = textureSample(screen_texture, texture_sampler, uv + offset).rgb;
            let lum = luminance(sample_color);

            // Quadratic falloff by distance
            let falloff = 1.0 - (r / radius);
            let falloff_sq = falloff * falloff;

            // Only accumulate from bright pixels
            if lum > 0.3 {
                glow_accum += lum * falloff_sq;
            }
        }

        // Normalize and apply glow
        let glow_normalized = glow_accum / f32(num_samples) * 4.0;
        let glow_color = vec3<f32>(0.0, 1.0, 0.667);  // #00ffaa cyan
        color = color + glow_color * glow_normalized * intensity;
    }

    // Apply scanlines
    if settings.scanlines_enabled != 0u {
        let screen_y = uv.y * settings.screen_size.y;
        // Create horizontal scanlines every 2 pixels
        let scanline = step(0.5, fract(screen_y * 0.5));
        // Make scanlines more visible by using higher opacity multiplier
        color = mix(color, color * 0.3, scanline * settings.scanline_opacity * 2.0);
    }

    // Apply flicker
    if settings.flicker_enabled != 0u {
        // Create irregular flicker pattern
        let flicker_seed = settings.time * 10.0;
        let flicker1 = step(0.95, random(flicker_seed));
        let flicker2 = step(0.90, random(flicker_seed + 0.5)) * 0.6;
        let flicker3 = step(0.85, random(flicker_seed + 1.0)) * 0.8;
        let flicker = max(max(flicker1, flicker2), flicker3);
        color = mix(color, vec3<f32>(0.0), flicker * settings.flicker_amount);
    }

    // Apply grid overlay
    if settings.grid_enabled != 0u {
        let screen_pos = uv * settings.screen_size;
        let grid_size = 40.0;
        let grid_line = max(
            step(0.98, fract(screen_pos.x / grid_size)),
            step(0.98, fract(screen_pos.y / grid_size))
        );

        // Apply vignette mask to grid when curvature is enabled
        var grid_opacity = 0.1;
        if settings.curvature_enabled != 0u {
            let centered = uv - 0.5;
            let dist = length(centered) * 2.0;
            let vignette = smoothstep(0.5, 1.0, dist);
            grid_opacity *= (1.0 - vignette);
        }

        // Grid color: rgba(0, 255, 170, 0.1) = #00ffaa
        let grid_color = vec3<f32>(0.0, 1.0, 0.667);
        color = mix(color, grid_color, grid_line * grid_opacity);
    }

    // Apply vignette when curvature is enabled
    if settings.curvature_enabled != 0u {
        let centered = uv - 0.5;
        let dist = length(centered) * 2.0;
        let vignette = smoothstep(0.8, 1.2, dist);
        color = mix(color, vec3<f32>(0.0), vignette); // fade to black
    }

    return vec4<f32>(color, 1.0);
}

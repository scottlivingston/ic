// CRT Post-Processing Shader (Second pass of separable Gaussian blur + CRT effects)
// Input: horizontally-blurred image from blur_horizontal.wgsl
// Implements: vertical blur completion, scanlines, flicker, curvature, grid

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

// Gaussian weight function
fn gaussian(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma));
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
        let edge_width = 0.02; // Width of the fade region
        let fade_x = smoothstep(0.0, edge_width, uv.x) * smoothstep(0.0, edge_width, 1.0 - uv.x);
        let fade_y = smoothstep(0.0, edge_width, uv.y) * smoothstep(0.0, edge_width, 1.0 - uv.y);
        edge_fade = fade_x * fade_y;

        // Still clip if completely outside bounds
        if uv.x < -0.01 || uv.x > 1.01 || uv.y < -0.01 || uv.y > 1.01 {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }

    // Sample the screen texture
    // RGB contains original color, alpha contains horizontal blur data
    let sample = textureSample(screen_texture, texture_sampler, uv);
    var color = sample.rgb;

    // Apply vertical blur (second pass of separable Gaussian blur)
    // Complete the blur by sampling alpha channel vertically
    if settings.glow_enabled != 0u {
        let pixel_size = 1.0 / settings.screen_size;

        var blur_accum = 0.0;
        var weight_sum = 0.0;

        let radius = 16;
        let sigma = 6.0;

        // Vertical blur pass - sample alpha channel (horizontal blur data) in Y direction
        for (var y = -radius; y <= radius; y++) {
            let offset = vec2<f32>(0.0, f32(y)) * pixel_size;
            let weight = gaussian(abs(f32(y)), sigma);
            let sample_alpha = textureSample(screen_texture, texture_sampler, uv + offset).a;

            blur_accum += sample_alpha * weight;
            weight_sum += weight;
        }

        // Complete the separable blur and apply glow with intensity
        let blur_normalized = blur_accum / weight_sum;
        let glow_color = vec3<f32>(0.0, 1.0, 0.667);  // #00ffaa cyan tint
        color = color + glow_color * blur_normalized * settings.glow_intensity;
    }

    // Apply scanlines
    if settings.scanlines_enabled != 0u {
        let screen_y = uv.y * settings.screen_size.y;
        // Create horizontal scanlines every 4 pixels (0.25 = 1/4)
        let scanline = step(0.5, fract(screen_y * 0.25));
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

        // Grid color: rgba(0, 255, 170, 0.1) = #00ffaa
        let grid_color = vec3<f32>(0.0, 1.0, 0.667);
        color = mix(color, grid_color, grid_line * 0.1);
    }

    // Apply edge fade for smooth antialiased edges on curvature
    color = color * edge_fade;

    return vec4<f32>(color, 1.0);
}

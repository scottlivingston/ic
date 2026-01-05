// CRT Post-Processing Shader
// Implements: glow, scanlines, flicker, curvature (barrel distortion), grid

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

// SDF for a box/rectangle
fn sdf_box(p: vec2<f32>, size: vec2<f32>) -> f32 {
    let d = abs(p) - size;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
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

    // Apply glow using SDF (perfectly smooth, no blur needed)
    if settings.glow_enabled != 0u {
        let intensity = settings.glow_intensity;

        // Convert UV to centered pixel coordinates (flip Y to match Bevy's coordinate system)
        let aspect = settings.screen_size.x / settings.screen_size.y;
        let p = (uv - 0.5) * vec2(settings.screen_size.x, -settings.screen_size.y);

        // Face element positions and sizes (matching face.rs)
        let eye_size = vec2(20.0, 40.0);      // half-size for SDF
        let mouth_size = vec2(30.0, 6.0);      // half-size for SDF
        // Golden ratio: face at Y=57 from center
        let left_eye_pos = vec2(-110.0, 57.0);
        let right_eye_pos = vec2(110.0, 57.0);
        let mouth_pos = vec2(0.0, 17.0);  // 57 - 40 (half eye height)

        // Calculate SDF for each element
        let d_left_eye = sdf_box(p - left_eye_pos, eye_size);
        let d_right_eye = sdf_box(p - right_eye_pos, eye_size);
        let d_mouth = sdf_box(p - mouth_pos, mouth_size);

        // Combine: use minimum distance to any shape
        let d = min(min(d_left_eye, d_right_eye), d_mouth);

        // Only apply glow outside the shapes (d > 0)
        // Steep exponential falloff
        let dist = max(d, 0.0);
        let falloff_rate = 0.21 / (intensity + 0.1);
        let glow = exp(-dist * falloff_rate) * 0.5;

        // Add glow with cyan tint (#00ffaa)
        let glow_color = vec3<f32>(0.0, 1.0, 0.667);
        color = color + glow_color * glow * intensity;
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

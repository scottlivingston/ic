// CRT Pixel Face Shader
// Renders face from bit array, then applies glow, scanlines, flicker, curvature

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct PixelFaceSettings {
    // CRT effects
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
    // Pixel face
    pixel_size: f32,
    // Padding to align face_pattern at 16-byte boundary (offset 64)
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    // 40x30 grid = 1200 bits = 40 u32s, packed as 10 vec4<u32> for 16-byte alignment
    face_pattern: array<vec4<u32>, 10>,
}

@group(0) @binding(2) var<uniform> settings: PixelFaceSettings;

const GRID_WIDTH: u32 = 40u;
const GRID_HEIGHT: u32 = 30u;
const FACE_COLOR: vec3<f32> = vec3<f32>(0.8, 1.0, 0.93);  // #ccffee

fn get_face_pixel(col: u32, row: u32) -> bool {
    let bit_index = row * GRID_WIDTH + col;
    let word_index = bit_index / 32u;
    let bit_offset = bit_index % 32u;
    // face_pattern is array<vec4<u32>, 10>, so we need to index into the vec4
    let vec_index = word_index / 4u;
    let vec_component = word_index % 4u;
    let word = settings.face_pattern[vec_index][vec_component];
    return (word & (1u << bit_offset)) != 0u;
}

fn render_pixel_face(screen_pos: vec2<f32>) -> vec3<f32> {
    let col = u32(screen_pos.x / settings.pixel_size);
    // Row 0 = top of screen (no Y flip)
    let row = u32(screen_pos.y / settings.pixel_size);

    if col >= GRID_WIDTH || row >= GRID_HEIGHT {
        return vec3<f32>(0.0);
    }

    if get_face_pixel(col, row) {
        return FACE_COLOR;
    }
    return vec3<f32>(0.0);
}

// Compute minimum distance to any lit pixel (for glow)
fn sdf_pixel_face(p: vec2<f32>) -> f32 {
    var min_dist = 99999.0;
    let pixel_size = settings.pixel_size;
    let half_pixel = pixel_size * 0.5;

    // Check distance to each lit pixel
    for (var row = 0u; row < GRID_HEIGHT; row++) {
        for (var col = 0u; col < GRID_WIDTH; col++) {
            if get_face_pixel(col, row) {
                // Center of this virtual pixel in screen coords (row 0 = top)
                let px = f32(col) * pixel_size + half_pixel;
                let py = f32(row) * pixel_size + half_pixel;

                // Distance to this pixel's box
                let d = abs(p - vec2(px, py)) - vec2(half_pixel);
                let dist = length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
                min_dist = min(min_dist, dist);
            }
        }
    }
    return min_dist;
}

fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let dist_sq = dot(centered, centered);
    return uv + centered * dist_sq * amount * 0.5;
}

fn random(seed: f32) -> f32 {
    return fract(sin(seed * 12.9898) * 43758.5453);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    var edge_fade = 1.0;

    // Passthrough region for diagnostics overlay (top-left corner)
    // UV (0,0) is top-left, so check for small x and y values
    if uv.x < 0.35 && uv.y < 0.35 {
        return textureSample(screen_texture, texture_sampler, uv);
    }

    // Barrel distortion (curvature)
    if settings.curvature_enabled != 0u {
        uv = barrel_distort(uv, settings.curvature_amount * 0.01);
        let edge_width = 0.02;
        let fade_x = smoothstep(0.0, edge_width, uv.x) * smoothstep(0.0, edge_width, 1.0 - uv.x);
        let fade_y = smoothstep(0.0, edge_width, uv.y) * smoothstep(0.0, edge_width, 1.0 - uv.y);
        edge_fade = fade_x * fade_y;

        if uv.x < -0.01 || uv.x > 1.01 || uv.y < -0.01 || uv.y > 1.01 {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }

    let screen_pos = uv * settings.screen_size;

    // Render pixel face
    var color = render_pixel_face(screen_pos);

    // Scanlines
    if settings.scanlines_enabled != 0u {
        let screen_y = uv.y * settings.screen_size.y;
        let scanline = step(0.5, fract(screen_y * 0.25));
        color = mix(color, color * 0.3, scanline * settings.scanline_opacity * 2.0);
    }

    // Glow disabled for Pi Zero 2 performance - the SDF loop is O(1200) per fragment
    // TODO: Replace with pre-computed distance field or neighbor-based approximation
    // if settings.glow_enabled != 0u {
    //     let d = sdf_pixel_face(screen_pos);
    //     let dist = max(d, 0.0);
    //     let falloff_rate = 0.21 / (settings.glow_intensity + 0.1);
    //     let glow = exp(-dist * falloff_rate) * 0.5;
    //     let glow_color = vec3<f32>(0.0, 1.0, 0.667);
    //     color = color + glow_color * glow * settings.glow_intensity;
    // }

    // Flicker
    if settings.flicker_enabled != 0u {
        let flicker_seed = settings.time * 10.0;
        let flicker1 = step(0.95, random(flicker_seed));
        let flicker2 = step(0.90, random(flicker_seed + 0.5)) * 0.6;
        let flicker3 = step(0.85, random(flicker_seed + 1.0)) * 0.8;
        let flicker = max(max(flicker1, flicker2), flicker3);
        color = mix(color, vec3<f32>(0.0), flicker * settings.flicker_amount);
    }

    // Grid overlay
    if settings.grid_enabled != 0u {
        let grid_size = 40.0;
        let grid_line = max(
            step(0.98, fract(screen_pos.x / grid_size)),
            step(0.98, fract(screen_pos.y / grid_size))
        );
        let grid_opacity = 0.1;
        let grid_color = vec3<f32>(0.0, 1.0, 0.667);
        color = mix(color, grid_color, grid_line * grid_opacity);
    }

    // Vignette - dim pixels at edges (uses barrel-distorted UVs to follow curvature)
    if settings.curvature_enabled != 0u {
        let centered = uv - 0.5;
        let dist = length(centered) * 2.0;
        // Start at 75% toward edge, full at 105% (wider gradient)
        let vignette = smoothstep(0.75, 1.05, dist);
        // Dim by up to 75% at edges
        color = color * (1.0 - vignette * 0.75);
    }

    color = color * edge_fade;
    return vec4<f32>(color, 1.0);
}

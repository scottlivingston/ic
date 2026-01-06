// Horizontal Gaussian Blur Pass
// First pass of separable blur - blurs bright pixels horizontally

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct BlurSettings {
    screen_size: vec2<f32>,
    intensity: f32,
    enabled: u32,
}

@group(0) @binding(2) var<uniform> settings: BlurSettings;

// Get luminance of a color
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3(0.299, 0.587, 0.114));
}

// Gaussian weight function
fn gaussian(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let color = textureSample(screen_texture, texture_sampler, uv).rgb;

    // If blur disabled, pass through with zero blur data in alpha
    if settings.enabled == 0u {
        return vec4<f32>(color, 0.0);
    }

    let pixel_size = 1.0 / settings.screen_size;
    let radius = 16;
    let sigma = 6.0;

    var blur_accum = 0.0;
    var weight_sum = 0.0;

    // Horizontal blur pass - sample only in X direction
    // Extract luminance of bright pixels and blur horizontally
    for (var x = -radius; x <= radius; x++) {
        let offset = vec2<f32>(f32(x), 0.0) * pixel_size;
        let weight = gaussian(abs(f32(x)), sigma);
        let sample_color = textureSample(screen_texture, texture_sampler, uv + offset).rgb;
        let lum = luminance(sample_color);

        // Only blur bright pixels (bloom threshold)
        if lum > 0.3 {
            blur_accum += lum * weight;
        }
        weight_sum += weight;
    }

    // Output: original color in RGB, horizontal blur luminance in alpha
    // The vertical pass will complete the blur and apply glow
    let blur_normalized = blur_accum / weight_sum;

    return vec4<f32>(color, blur_normalized);
}

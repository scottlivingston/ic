// Additive Glow Shader with Atlas Support
// Samples from a region of the glow atlas texture using UV rect
// Uses additive blending (set in Material2d::specialize)

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var glow_texture: texture_2d<f32>;
@group(2) @binding(1) var glow_sampler: sampler;

struct GlowUniforms {
    intensity: f32,
    // UV rect: (min_u, min_v, max_u, max_v) for atlas sampling
    uv_rect: vec4<f32>,
}
@group(2) @binding(2) var<uniform> uniforms: GlowUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Remap UV from [0,1] to the atlas rect region
    let atlas_uv = mix(uniforms.uv_rect.xy, uniforms.uv_rect.zw, in.uv);

    let color = textureSample(glow_texture, glow_sampler, atlas_uv);
    // Multiply RGB by intensity, keep alpha for blend control
    return vec4<f32>(color.rgb * uniforms.intensity, color.a);
}

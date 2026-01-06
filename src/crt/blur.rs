//! Horizontal blur pass for separable Gaussian blur

#![allow(dead_code)] // ShaderType derive generates unused check functions

use bevy::{
    core_pipeline::FullscreenShader,
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, Buffer, BufferDescriptor,
            BufferUsages, CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState,
            Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::ViewTarget,
    },
};

use super::CrtSettings;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct BlurHorizontalLabel;

/// Uniform for blur shader
#[derive(ShaderType, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct BlurSettingsUniform {
    screen_size: Vec2,
    intensity: f32,
    enabled: u32,
}

impl From<&CrtSettings> for BlurSettingsUniform {
    fn from(settings: &CrtSettings) -> Self {
        Self {
            screen_size: Vec2::new(settings.screen_width, settings.screen_height),
            intensity: settings.glow_intensity,
            enabled: settings.glow_enabled,
        }
    }
}

#[derive(Default)]
pub struct BlurHorizontalNode;

impl ViewNode for BlurHorizontalNode {
    type ViewQuery = (&'static ViewTarget, &'static CrtSettings);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings): bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let blur_pipeline = world.resource::<BlurPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(blur_pipeline.pipeline_id) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        // Update uniform buffer
        let uniform = BlurSettingsUniform::from(settings);
        let render_queue = world.resource::<RenderQueue>();
        render_queue.write_buffer(&blur_pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let bind_group = render_context.render_device().create_bind_group(
            "blur_horizontal_bind_group",
            &blur_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &blur_pipeline.sampler,
                blur_pipeline.uniform_buffer.as_entire_binding(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("blur_horizontal_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub struct BlurPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_buffer: Buffer,
}

impl BlurPipeline {
    pub fn new(
        render_device: &RenderDevice,
        asset_server: &AssetServer,
        pipeline_cache: &PipelineCache,
        fullscreen_shader: &FullscreenShader,
    ) -> Self {
        let layout = render_device.create_bind_group_layout(
            "blur_horizontal_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<BlurSettingsUniform>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("blur_settings_buffer"),
            size: std::mem::size_of::<BlurSettingsUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = asset_server.load::<Shader>("embedded://ic/crt/shaders/blur_horizontal.wgsl");

        let pipeline_id = pipeline_cache.queue_render_pipeline(
            RenderPipelineDescriptor {
                label: Some("blur_horizontal_pipeline".into()),
                layout: vec![layout.clone()],
                vertex: fullscreen_shader.to_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::Rgba8UnormSrgb,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                    ..default()
                }),
                ..default()
            },
        );

        Self {
            layout,
            sampler,
            pipeline_id,
            uniform_buffer,
        }
    }
}

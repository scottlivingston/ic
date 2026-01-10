#![allow(dead_code)] // ShaderType derive generates unused check functions

pub mod effects;

use bevy::{
    asset::embedded_asset,
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        FullscreenShader,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
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
        RenderApp, RenderStartup,
    },
};

use effects::{update_effects_from_events, CrtEffects};

use crate::face::{EYE_GAP, EYE_HEIGHT, EYE_WIDTH, FACE_Y_OFFSET, MOUTH_BASE_Y, MOUTH_HEIGHT, MOUTH_WIDTH, Mouth};

pub struct CrtPlugin;

impl Plugin for CrtPlugin {
    fn build(&self, app: &mut App) {
        // SDF shader (single pass, faster)
        embedded_asset!(app, "shaders/crt_sdf.wgsl");

        app.init_resource::<CrtEffects>()
            .add_plugins(ExtractComponentPlugin::<CrtSettings>::default())
            .add_systems(Update, update_effects_from_events)
            .add_systems(Update, sync_crt_settings);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_crt_pipeline)
            .add_render_graph_node::<ViewNodeRunner<CrtNode>>(Core2d, CrtLabel)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::Tonemapping,
                    CrtLabel,  // Single pass SDF-based CRT effects
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CrtLabel;

/// Component that holds CRT settings for the camera.
/// Boolean flags use u32 instead of bool for WGSL shader compatibility.
#[derive(Component, Default, Clone, Copy, ExtractComponent)]
pub struct CrtSettings {
    pub glow_enabled: u32,
    pub glow_intensity: f32,
    pub scanlines_enabled: u32,
    pub scanline_opacity: f32,
    pub flicker_enabled: u32,
    pub flicker_amount: f32,
    pub curvature_enabled: u32,
    pub curvature_amount: f32,
    pub grid_enabled: u32,
    pub time: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    // Face geometry for SDF glow
    pub left_eye_pos: Vec2,
    pub right_eye_pos: Vec2,
    pub eye_half_size: Vec2,
    pub mouth_pos: Vec2,
    pub mouth_half_size: Vec2,
}

impl CrtSettings {
    /// Apply effect settings from CrtEffects resource
    fn apply_from(&mut self, effects: &CrtEffects) {
        self.glow_enabled = effects.glow_enabled as u32;
        self.glow_intensity = effects.glow_intensity;
        self.scanlines_enabled = effects.scanlines_enabled as u32;
        self.scanline_opacity = effects.scanline_opacity;
        self.flicker_enabled = effects.flicker_enabled as u32;
        self.flicker_amount = effects.flicker_amount;
        self.curvature_enabled = effects.curvature_enabled as u32;
        self.curvature_amount = effects.curvature_amount;
        self.grid_enabled = effects.grid_enabled as u32;
    }
}

#[derive(ShaderType, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct CrtSettingsUniform {
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
    screen_size: Vec2,
    // Face geometry for SDF glow
    left_eye_pos: Vec2,
    right_eye_pos: Vec2,
    eye_half_size: Vec2,
    mouth_pos: Vec2,
    mouth_half_size: Vec2,
}

impl From<&CrtSettings> for CrtSettingsUniform {
    fn from(settings: &CrtSettings) -> Self {
        Self {
            glow_enabled: settings.glow_enabled,
            glow_intensity: settings.glow_intensity,
            scanlines_enabled: settings.scanlines_enabled,
            scanline_opacity: settings.scanline_opacity,
            flicker_enabled: settings.flicker_enabled,
            flicker_amount: settings.flicker_amount,
            curvature_enabled: settings.curvature_enabled,
            curvature_amount: settings.curvature_amount,
            grid_enabled: settings.grid_enabled,
            time: settings.time,
            screen_size: Vec2::new(settings.screen_width, settings.screen_height),
            left_eye_pos: settings.left_eye_pos,
            right_eye_pos: settings.right_eye_pos,
            eye_half_size: settings.eye_half_size,
            mouth_pos: settings.mouth_pos,
            mouth_half_size: settings.mouth_half_size,
        }
    }
}

fn sync_crt_settings(
    crt_effects: Res<CrtEffects>,
    time: Res<Time>,
    windows: Query<&Window>,
    mut cameras: Query<&mut CrtSettings, With<Camera2d>>,
    mouth_query: Query<(&Sprite, &Transform), With<Mouth>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut settings) = cameras.single_mut() else {
        return;
    };

    settings.apply_from(&crt_effects);
    settings.time = time.elapsed_secs();
    settings.screen_width = window.width();
    settings.screen_height = window.height();

    // Face geometry for SDF glow (from face.rs constants)
    settings.left_eye_pos = Vec2::new(-EYE_GAP / 2.0 - MOUTH_WIDTH / 2.0, FACE_Y_OFFSET);
    settings.right_eye_pos = Vec2::new(EYE_GAP / 2.0 + MOUTH_WIDTH / 2.0, FACE_Y_OFFSET);
    settings.eye_half_size = Vec2::new(EYE_WIDTH / 2.0, EYE_HEIGHT / 2.0);

    // Get mouth geometry from actual sprite (follows animation)
    if let Ok((sprite, transform)) = mouth_query.single() {
        let size = sprite.custom_size.unwrap_or(Vec2::new(MOUTH_WIDTH, MOUTH_HEIGHT));
        settings.mouth_pos = transform.translation.truncate();
        settings.mouth_half_size = size / 2.0;
    } else {
        // Fallback to static values if mouth not found
        settings.mouth_pos = Vec2::new(0.0, MOUTH_BASE_Y);
        settings.mouth_half_size = Vec2::new(MOUTH_WIDTH / 2.0, MOUTH_HEIGHT / 2.0);
    }
}

#[derive(Default)]
struct CrtNode;

impl ViewNode for CrtNode {
    type ViewQuery = (&'static ViewTarget, &'static CrtSettings);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings): bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let crt_pipeline = world.resource::<CrtPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(crt_pipeline.pipeline_id) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        // Update cached uniform buffer with current settings
        let uniform = CrtSettingsUniform::from(settings);
        let render_queue = world.resource::<RenderQueue>();
        render_queue.write_buffer(&crt_pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let bind_group = render_context.render_device().create_bind_group(
            "crt_bind_group",
            &crt_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &crt_pipeline.sampler,
                crt_pipeline.uniform_buffer.as_entire_binding(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("crt_pass"),
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
struct CrtPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
    uniform_buffer: Buffer,
}

impl CrtPipeline {
    fn new(
        render_device: &RenderDevice,
        asset_server: &AssetServer,
        pipeline_cache: &PipelineCache,
        fullscreen_shader: &FullscreenShader,
    ) -> Self {
        let layout = render_device.create_bind_group_layout(
            "crt_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<CrtSettingsUniform>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Create cached uniform buffer (will be updated each frame)
        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("crt_settings_buffer"),
            size: std::mem::size_of::<CrtSettingsUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = asset_server.load::<Shader>("embedded://ic/crt/shaders/crt_sdf.wgsl");

        let pipeline_id = pipeline_cache.queue_render_pipeline(
            RenderPipelineDescriptor {
                label: Some("crt_pipeline".into()),
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

fn init_crt_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    fullscreen_shader: Res<FullscreenShader>,
) {
    commands.insert_resource(CrtPipeline::new(
        &render_device,
        &asset_server,
        &pipeline_cache,
        &fullscreen_shader,
    ));
}

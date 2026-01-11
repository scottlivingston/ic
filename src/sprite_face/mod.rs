//! Sprite Face Plugin - renders face from pre-generated atlas images
//!
//! Uses pre-rendered PNG atlases for efficient rendering on devices with
//! software rendering (like Pi Zero 2 W). Atlas-based approach eliminates
//! texture swapping overhead that caused 2fps on Pi Zero 2.
//!
//! Only barrel distortion is computed at runtime via a minimal post-process shader.

use bevy::{
    asset::embedded_asset,
    core_pipeline::{
        FullscreenShader,
        core_2d::graph::{Core2d, Node2d},
    },
    image::TextureAtlasLayout,
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            AsBindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries,
            BlendComponent, BlendFactor, BlendOperation, BlendState,
            Buffer, BufferDescriptor, BufferUsages, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FragmentState, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::ViewTarget,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dKey, Material2dPlugin, MeshMaterial2d},
};

use bevy::render::render_resource::SpecializedMeshPipelineError;
use bevy_mesh::MeshVertexBufferLayoutRef;

use crate::app_state::AppMode;
use crate::crt::effects::{CrtEffects, update_effects_from_events};
use crate::diagnostics::SystemTimings;
use crate::events::FaceType;
use crate::face::SpeakingState;

/// Tracks current face animation state to avoid unnecessary GPU updates
#[derive(Resource, Default)]
struct FaceAnimationState {
    current_index: usize,
    // Track last known effect values to avoid redundant updates
    last_glow_intensity: f32,
    last_glow_enabled: bool,
}

// ============================================================================
// Glow Material (Additive Blending with Atlas UV support)
// ============================================================================

/// UV rects for each atlas cell (2x2 grid)
/// Index: 0=default, 1=talking, 2=angry, 3=angry_talking
const ATLAS_UV_RECTS: [Vec4; 4] = [
    Vec4::new(0.0, 0.0, 0.5, 0.5),   // 0: top-left (default)
    Vec4::new(0.5, 0.0, 1.0, 0.5),   // 1: top-right (talking)
    Vec4::new(0.0, 0.5, 0.5, 1.0),   // 2: bottom-left (angry)
    Vec4::new(0.5, 0.5, 1.0, 1.0),   // 3: bottom-right (angry_talking)
];

/// Custom material for glow with additive blending and atlas UV support
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GlowMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub uniforms: GlowUniforms,
}

/// Uniforms for glow shader - must match WGSL struct layout
#[derive(Clone, Copy, Default, ShaderType)]
pub struct GlowUniforms {
    pub intensity: f32,
    pub uv_rect: Vec4, // (min_u, min_v, max_u, max_v)
}

use bevy::sprite_render::AlphaMode2d;

impl Material2d for GlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://ic/sprite_face/shaders/glow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Set additive blending: src + dst (glow adds to underlying pixels)
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target) = fragment.targets.get_mut(0).and_then(|t| t.as_mut()) {
                target.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                });
            }
        }
        Ok(())
    }
}

// ============================================================================
// Plugin
// ============================================================================

pub struct SpriteFacePlugin;

impl Plugin for SpriteFacePlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/crt_simple.wgsl");
        embedded_asset!(app, "shaders/glow.wgsl");
        // Face sprite assets are embedded via OnboardingPlugin

        app.init_resource::<CrtEffects>()
            .init_resource::<SpeakingState>()
            .init_resource::<FaceAnimationState>()
            .add_plugins(Material2dPlugin::<GlowMaterial>::default())
            .add_plugins(ExtractComponentPlugin::<CrtSimpleSettings>::default())
            .add_systems(Startup, load_face_textures)
            .add_systems(OnEnter(AppMode::Normal), spawn_face_sprites)
            .add_systems(OnExit(AppMode::Normal), despawn_face_sprites)
            .add_systems(Update, update_effects_from_events)
            .add_systems(Update, debug_asset_loading)
            .add_systems(
                Update,
                (sync_crt_settings, update_face_sprites, update_sprite_effects)
                    .run_if(in_state(AppMode::Normal)),
            );

        // CRT post-process disabled for Pi Zero 2 performance testing
        // The fullscreen pass takes 250ms even with all effects disabled
        // TODO: Re-enable for desktop builds or when Pi performance is solved
        //
        // let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        //     return;
        // };
        //
        // render_app
        //     .add_systems(RenderStartup, init_crt_simple_pipeline)
        //     .add_render_graph_node::<ViewNodeRunner<CrtSimpleNode>>(Core2d, CrtSimpleLabel)
        //     .add_render_graph_edges(
        //         Core2d,
        //         (
        //             Node2d::Tonemapping,
        //             CrtSimpleLabel,
        //             Node2d::EndMainPassPostProcessing,
        //         ),
        //     );
    }
}

// ============================================================================
// Face Textures Resource (Atlas-based for performance)
// ============================================================================

/// Atlas index mapping:
///   0 = default (closed mouth)
///   1 = talking (open mouth)
///   2 = angry (closed)
///   3 = angry_talking (open)
#[derive(Resource, Default)]
struct FaceTextures {
    base_atlas: Handle<Image>,
    glow_atlas: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    scanlines: Handle<Image>,
    grid: Handle<Image>,
}

fn load_face_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load from embedded assets (embedded via OnboardingPlugin)
    let base = "embedded://ic/assets/sprites";
    info!("Loading face atlases from {}", base);

    // Create atlas layout: 2x2 grid of 640x480 cells
    let layout = TextureAtlasLayout::from_grid(UVec2::new(640, 480), 2, 2, None, None);
    let layout_handle = layouts.add(layout);

    commands.insert_resource(FaceTextures {
        base_atlas: asset_server.load(format!("{}/face_base_atlas.png", base)),
        glow_atlas: asset_server.load(format!("{}/face_glow_atlas.png", base)),
        atlas_layout: layout_handle,
        scanlines: asset_server.load(format!("{}/scanlines.png", base)),
        grid: asset_server.load(format!("{}/grid.png", base)),
    });
}

fn debug_asset_loading(
    asset_server: Res<AssetServer>,
    textures: Res<FaceTextures>,
    mut logged: Local<bool>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;

    // Log every 60 frames (about once per second)
    if *frame_count % 60 == 1 || *logged {
        if *logged {
            return;
        }

        use bevy::asset::LoadState;

        let state = asset_server.get_load_state(&textures.base_atlas);
        info!("Frame {}: Atlas load state = {:?}", *frame_count, state);

        match state {
            Some(LoadState::Loaded) => {
                info!("face_base_atlas loaded successfully!");
                *logged = true;
            }
            Some(LoadState::Failed(err)) => {
                error!("face_base_atlas FAILED to load: {:?}", err);
                *logged = true;
            }
            _ => {}
        }
    }
}

// ============================================================================
// Face Sprites
// ============================================================================

#[derive(Component)]
struct FaceBaseSprite;

#[derive(Component)]
struct FaceGlowSprite;

#[derive(Component)]
struct ScanlineSprite;

#[derive(Component)]
struct GridSprite;

fn spawn_face_sprites(
    mut commands: Commands,
    textures: Res<FaceTextures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut glow_materials: ResMut<Assets<GlowMaterial>>,
) {
    info!("Spawning face sprites with atlas");

    // Base face layer (bottom, z=0) - using texture atlas
    commands.spawn((
        FaceBaseSprite,
        Sprite {
            image: textures.base_atlas.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: textures.atlas_layout.clone(),
                index: 0, // Start with default (closed mouth)
            }),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Scanlines overlay (z=1)
    commands.spawn((
        ScanlineSprite,
        Sprite {
            image: textures.scanlines.clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    // Grid overlay (z=1.5, starts hidden)
    commands.spawn((
        GridSprite,
        Sprite {
            image: textures.grid.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, 0.0), // Start invisible
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.5),
    ));

    // Glow layer disabled for Pi Zero 2 performance testing
    // TODO: Re-enable when performance is solved
    //
    // let glow_mesh = meshes.add(Rectangle::new(640.0, 480.0));
    // let glow_material = glow_materials.add(GlowMaterial {
    //     texture: textures.glow_atlas.clone(),
    //     uniforms: GlowUniforms {
    //         intensity: 1.0,
    //         uv_rect: ATLAS_UV_RECTS[0], // Start with default face
    //     },
    // });
    // commands.spawn((
    //     FaceGlowSprite,
    //     Mesh2d(glow_mesh),
    //     MeshMaterial2d(glow_material),
    //     Transform::from_xyz(0.0, 0.0, 2.0),
    // ));

    info!("Face sprites spawned with atlas");
}

fn despawn_face_sprites(
    mut commands: Commands,
    base_query: Query<Entity, With<FaceBaseSprite>>,
    glow_query: Query<Entity, With<FaceGlowSprite>>,
    scanline_query: Query<Entity, With<ScanlineSprite>>,
    grid_query: Query<Entity, With<GridSprite>>,
) {
    for entity in base_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in glow_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in scanline_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in grid_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Atlas index mapping:
///   0 = default (closed mouth)
///   1 = talking (open mouth)
///   2 = angry (closed)
///   3 = angry_talking (open)
fn update_face_sprites(
    time: Res<Time>,
    speaking: Res<SpeakingState>,
    mut timings: ResMut<SystemTimings>,
    mut anim_state: ResMut<FaceAnimationState>,
    mut base_query: Query<&mut Sprite, With<FaceBaseSprite>>,
    glow_query: Query<&MeshMaterial2d<GlowMaterial>, With<FaceGlowSprite>>,
    mut glow_materials: ResMut<Assets<GlowMaterial>>,
) {
    let start = std::time::Instant::now();

    // Determine atlas index based on speaking state and face type
    let atlas_index = if speaking.speaking {
        // Alternate between open/closed mouth at 12fps
        let frame = (time.elapsed_secs() * 12.0) as u32 % 2;
        match speaking.face_type {
            FaceType::Default => {
                if frame == 0 { 1 } else { 0 } // talking / default
            }
            FaceType::Angry => {
                if frame == 0 { 3 } else { 2 } // angry_talking / angry
            }
        }
    } else {
        0 // default (closed mouth)
    };

    // Only update GPU resources if the face actually changed
    // This avoids unnecessary uniform buffer updates that cause GPU stalls
    if atlas_index != anim_state.current_index {
        anim_state.current_index = atlas_index;

        // Update base sprite atlas index
        for mut sprite in base_query.iter_mut() {
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = atlas_index;
            }
        }

        // Update glow material UV rect
        for material_handle in glow_query.iter() {
            if let Some(material) = glow_materials.get_mut(&material_handle.0) {
                material.uniforms.uv_rect = ATLAS_UV_RECTS[atlas_index];
            }
        }
    }

    timings.face_update_ms = start.elapsed().as_secs_f32() * 1000.0;
}

fn update_sprite_effects(
    effects: Res<CrtEffects>,
    mut timings: ResMut<SystemTimings>,
    mut anim_state: ResMut<FaceAnimationState>,
    glow_query: Query<&MeshMaterial2d<GlowMaterial>, With<FaceGlowSprite>>,
    mut glow_materials: ResMut<Assets<GlowMaterial>>,
    mut scanline_query: Query<&mut Sprite, (With<ScanlineSprite>, Without<GridSprite>)>,
    mut grid_query: Query<&mut Sprite, (With<GridSprite>, Without<ScanlineSprite>)>,
) {
    let start = std::time::Instant::now();

    // Only update glow material if intensity or enabled state changed
    let glow_changed = effects.glow_enabled != anim_state.last_glow_enabled
        || effects.glow_intensity != anim_state.last_glow_intensity;

    if glow_changed {
        anim_state.last_glow_enabled = effects.glow_enabled;
        anim_state.last_glow_intensity = effects.glow_intensity;

        for material_handle in glow_query.iter() {
            if let Some(material) = glow_materials.get_mut(&material_handle.0) {
                material.uniforms.intensity = if effects.glow_enabled {
                    effects.glow_intensity * 4.0
                } else {
                    0.0
                };
            }
        }
    }

    // Scanlines and grid use Sprite color which is cheaper to update,
    // but we could optimize these too if needed
    for mut sprite in scanline_query.iter_mut() {
        let alpha = if effects.scanlines_enabled {
            effects.scanline_opacity
        } else {
            0.0
        };
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }

    for mut sprite in grid_query.iter_mut() {
        let alpha = if effects.grid_enabled { 0.15 } else { 0.0 };
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }

    timings.glow_update_ms = start.elapsed().as_secs_f32() * 1000.0;
}

// ============================================================================
// CRT Simple Post-Process Shader
// ============================================================================

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CrtSimpleLabel;

/// Component that holds CRT settings for the camera
#[derive(Component, Clone, Copy, ExtractComponent)]
pub struct CrtSimpleSettings {
    pub curvature_enabled: u32,
    pub curvature_amount: f32,
    pub flicker_enabled: u32,
    pub flicker_amount: f32,
    pub time: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Default for CrtSimpleSettings {
    fn default() -> Self {
        Self {
            // Disabled by default for Pi Zero 2 performance testing
            curvature_enabled: 0,
            curvature_amount: 50.0,
            flicker_enabled: 0,
            flicker_amount: 0.5,
            time: 0.0,
            screen_width: 640.0,
            screen_height: 480.0,
        }
    }
}

/// Uniform struct for shader - must match WGSL layout exactly
/// Vec2 requires 8-byte alignment, so we add explicit padding
#[derive(ShaderType, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct CrtSimpleSettingsUniform {
    curvature_enabled: u32,
    curvature_amount: f32,
    flicker_enabled: u32,
    flicker_amount: f32,
    time: f32,
    _padding: f32, // Align screen_size to 8-byte boundary
    screen_size: Vec2,
}

impl From<&CrtSimpleSettings> for CrtSimpleSettingsUniform {
    fn from(settings: &CrtSimpleSettings) -> Self {
        Self {
            curvature_enabled: settings.curvature_enabled,
            curvature_amount: settings.curvature_amount,
            flicker_enabled: settings.flicker_enabled,
            flicker_amount: settings.flicker_amount,
            time: settings.time,
            _padding: 0.0,
            screen_size: Vec2::new(settings.screen_width, settings.screen_height),
        }
    }
}

fn sync_crt_settings(
    time: Res<Time>,
    effects: Res<CrtEffects>,
    windows: Query<&Window>,
    mut cameras: Query<&mut CrtSimpleSettings, With<Camera2d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut settings) = cameras.single_mut() else {
        return;
    };

    settings.curvature_enabled = effects.curvature_enabled as u32;
    settings.curvature_amount = effects.curvature_amount;
    settings.flicker_enabled = effects.flicker_enabled as u32;
    settings.flicker_amount = effects.flicker_amount;
    settings.time = time.elapsed_secs();
    settings.screen_width = window.width();
    settings.screen_height = window.height();
}

// ============================================================================
// Render Pipeline
// ============================================================================

#[derive(Default)]
struct CrtSimpleNode;

impl ViewNode for CrtSimpleNode {
    type ViewQuery = (&'static ViewTarget, &'static CrtSimpleSettings);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings): bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<CrtSimplePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        // Update uniform buffer
        let uniform = CrtSimpleSettingsUniform::from(settings);
        let render_queue = world.resource::<RenderQueue>();
        render_queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let bind_group = render_context.render_device().create_bind_group(
            "crt_simple_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.sampler,
                pipeline.uniform_buffer.as_entire_binding(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("crt_simple_pass"),
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

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct CrtSimplePipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
    uniform_buffer: Buffer,
}

impl CrtSimplePipeline {
    fn new(
        render_device: &RenderDevice,
        asset_server: &AssetServer,
        pipeline_cache: &PipelineCache,
        fullscreen_shader: &FullscreenShader,
    ) -> Self {
        let layout = render_device.create_bind_group_layout(
            "crt_simple_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<CrtSimpleSettingsUniform>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("crt_simple_settings_buffer"),
            size: std::mem::size_of::<CrtSimpleSettingsUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader =
            asset_server.load::<Shader>("embedded://ic/sprite_face/shaders/crt_simple.wgsl");

        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("crt_simple_pipeline".into()),
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
        });

        Self {
            layout,
            sampler,
            pipeline_id,
            uniform_buffer,
        }
    }
}

fn init_crt_simple_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    fullscreen_shader: Res<FullscreenShader>,
) {
    commands.insert_resource(CrtSimplePipeline::new(
        &render_device,
        &asset_server,
        &pipeline_cache,
        &fullscreen_shader,
    ));
}

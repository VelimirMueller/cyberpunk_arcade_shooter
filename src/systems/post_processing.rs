use bevy::app::{App, Plugin};
use bevy::asset::{Assets, Handle, weak_handle};
use bevy::core_pipeline::{
    core_2d::graph::{Core2d, Node2d},
    fullscreen_vertex_shader::fullscreen_shader_vertex_state,
};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{QueryItem, With},
    resource::Resource,
    schedule::IntoScheduleConfigs as _,
    system::{Commands, Query, Res, ResMut, lifetimeless::Read},
    world::{FromWorld, World},
};
use bevy::image::BevyDefault;
use bevy::prelude::Camera;
use bevy::render::{
    Render, RenderApp, RenderSet,
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    render_graph::{
        NodeRunError, RenderGraphApp as _, RenderGraphContext, RenderLabel, ViewNode,
        ViewNodeRunner,
    },
    render_resource::{
        BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
        ColorTargetState, ColorWrites, DynamicUniformBuffer, FilterMode, FragmentState, Operations,
        PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
        Sampler, SamplerBindingType, SamplerDescriptor, Shader, ShaderStages, ShaderType,
        SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, TextureSampleType,
        binding_types::{sampler, texture_2d, uniform_buffer},
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    view::{ExtractedView, ViewTarget},
};
use bevy::utils::prelude::default;

// ---------------------------------------------------------------------------
// Render graph label
// ---------------------------------------------------------------------------

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct CrtPostProcessLabel;

// ---------------------------------------------------------------------------
// Main-world component (attached to camera)
// ---------------------------------------------------------------------------

#[derive(Component, Clone)]
pub struct CrtSettings {
    pub scanline_intensity: f32,
    pub scanline_count: f32,
    pub vignette_intensity: f32,
    pub vignette_radius: f32,
    pub curvature_amount: f32,
}

impl Default for CrtSettings {
    fn default() -> Self {
        Self {
            scanline_intensity: 0.15,
            scanline_count: 200.0,
            vignette_intensity: 0.4,
            vignette_radius: 0.7,
            curvature_amount: 0.02,
        }
    }
}

// Manual ExtractComponent impl (mirrors ChromaticAberration pattern)
impl ExtractComponent for CrtSettings {
    type QueryData = Read<CrtSettings>;
    type QueryFilter = With<Camera>;
    type Out = CrtSettings;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some(item.clone())
    }
}

// ---------------------------------------------------------------------------
// GPU uniform (must be 16-byte aligned to match WGSL struct)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Clone, Copy, ShaderType)]
struct CrtSettingsUniform {
    scanline_intensity: f32,
    scanline_count: f32,
    vignette_intensity: f32,
    vignette_radius: f32,
    curvature_amount: f32,
    _padding: f32,
    _padding2: f32,
    _padding3: f32,
}

// ---------------------------------------------------------------------------
// Render-world resources
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct CrtUniformBuffers {
    buffer: DynamicUniformBuffer<CrtSettingsUniform>,
}

#[derive(Component)]
struct CrtUniformBufferOffset(u32);

#[derive(Component)]
struct CrtPipelineId(CachedRenderPipelineId);

#[derive(Resource)]
struct CrtPipeline {
    bind_group_layout: BindGroupLayout,
    source_sampler: Sampler,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct CrtPipelineKey {
    texture_format: TextureFormat,
}

// ---------------------------------------------------------------------------
// Pipeline construction (FromWorld)
// ---------------------------------------------------------------------------

impl FromWorld for CrtPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = render_device.create_bind_group_layout(
            Some("crt_post_process_bind_group_layout"),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // @binding(0) screen_texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // @binding(1) texture_sampler
                    sampler(SamplerBindingType::Filtering),
                    // @binding(2) settings uniform
                    uniform_buffer::<CrtSettingsUniform>(true),
                ),
            ),
        );

        let source_sampler = render_device.create_sampler(&SamplerDescriptor {
            mipmap_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            ..default()
        });

        CrtPipeline {
            bind_group_layout,
            source_sampler,
        }
    }
}

impl SpecializedRenderPipeline for CrtPipeline {
    type Key = CrtPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("crt_post_process_pipeline".into()),
            layout: vec![self.bind_group_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: CRT_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: default(),
            depth_stencil: None,
            multisample: default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Shader handle
// ---------------------------------------------------------------------------

const CRT_SHADER_HANDLE: Handle<Shader> = weak_handle!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");

// ---------------------------------------------------------------------------
// Render node
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CrtPostProcessNode;

impl ViewNode for CrtPostProcessNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<CrtPipelineId>,
        Read<CrtSettings>,
        Read<CrtUniformBufferOffset>,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_target, pipeline_id, _settings, uniform_offset): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let crt_pipeline = world.resource::<CrtPipeline>();
        let uniform_buffers = world.resource::<CrtUniformBuffers>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
            return Ok(());
        };

        let Some(uniform_binding) = uniform_buffers.buffer.binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            Some("crt_post_process_bind_group"),
            &crt_pipeline.bind_group_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &crt_pipeline.source_sampler,
                uniform_binding,
            )),
        );

        let mut render_pass =
            render_context
                .command_encoder()
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("crt_post_process_pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: post_process.destination,
                        resolve_target: None,
                        ops: Operations::default(),
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[uniform_offset.0]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Prepare systems (run in render world)
// ---------------------------------------------------------------------------

fn prepare_crt_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CrtPipeline>>,
    crt_pipeline: Res<CrtPipeline>,
    views: Query<(Entity, &ExtractedView), With<CrtSettings>>,
) {
    for (entity, view) in views.iter() {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &crt_pipeline,
            CrtPipelineKey {
                texture_format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
            },
        );
        commands.entity(entity).insert(CrtPipelineId(pipeline_id));
    }
}

fn prepare_crt_uniforms(
    mut commands: Commands,
    mut uniform_buffers: ResMut<CrtUniformBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    views: Query<(Entity, &CrtSettings)>,
) {
    uniform_buffers.buffer.clear();

    for (entity, settings) in views.iter() {
        let offset = uniform_buffers.buffer.push(&CrtSettingsUniform {
            scanline_intensity: settings.scanline_intensity,
            scanline_count: settings.scanline_count,
            vignette_intensity: settings.vignette_intensity,
            vignette_radius: settings.vignette_radius,
            curvature_amount: settings.curvature_amount,
            _padding: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
        });
        commands
            .entity(entity)
            .insert(CrtUniformBufferOffset(offset));
    }

    uniform_buffers
        .buffer
        .write_buffer(&render_device, &render_queue);
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CrtPostProcessPlugin;

impl Plugin for CrtPostProcessPlugin {
    fn build(&self, app: &mut App) {
        // Load the shader as an asset so the render pipeline can find it.
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
        shaders.insert(
            &CRT_SHADER_HANDLE,
            Shader::from_wgsl(
                include_str!("../../assets/shaders/crt_post_process.wgsl"),
                file!(),
            ),
        );

        app.add_plugins(ExtractComponentPlugin::<CrtSettings>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<CrtPipeline>>()
            .init_resource::<CrtUniformBuffers>()
            .add_systems(
                Render,
                (prepare_crt_pipelines, prepare_crt_uniforms).in_set(RenderSet::Prepare),
            )
            .add_render_graph_node::<ViewNodeRunner<CrtPostProcessNode>>(
                Core2d,
                CrtPostProcessLabel,
            )
            .add_render_graph_edges(
                Core2d,
                (Node2d::Tonemapping, CrtPostProcessLabel, Node2d::Fxaa),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<CrtPipeline>();
    }
}

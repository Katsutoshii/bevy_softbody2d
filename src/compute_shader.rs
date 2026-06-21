use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bevy::{
    app::{App, Plugin},
    ecs::{resource::Resource, schedule::SystemCondition},
    render::{RenderSystems, alpha::AlphaMode, render_resource::BindGroupLayoutDescriptor},
    shader::{Shader, ShaderDefVal, ShaderRef},
};
use bevy::{
    asset::DirectAssetAccessExt,
    asset::Handle,
    ecs::{
        schedule::{
            IntoScheduleConfigs,
            common_conditions::{
                not, resource_changed, resource_exists, resource_exists_and_changed,
            },
        },
        system::{Commands, Res, ResMut, StaticSystemParam},
        world::{FromWorld, World},
    },
    math::UVec3,
    render::{
        ExtractSchedule, MainWorld, Render, RenderApp,
        extract_resource::{ExtractResource, ExtractResourcePlugin, extract_resource},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            AsBindGroup, BindGroup, CachedComputePipelineId, CachedPipelineState,
            ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache,
        },
        renderer::{RenderContext, RenderDevice},
    },
    state::{
        app::AppExtStates,
        state::{NextState, States},
    },
};

/// Plugin to create all the required systems for using a custom compute shader.
pub struct ComputeShaderPlugin<S: ComputeShader> {
    pub _marker: PhantomData<S>,
}
impl<S: ComputeShader> Default for ComputeShaderPlugin<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> Plugin for ComputeShaderPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<S>()
            .add_plugins(ExtractResourcePlugin::<S>::default())
            .init_state::<ComputeNodeState<S>>();
    }

    fn finish(&self, app: &mut App) {
        // Add the compute shader resources and systems to the render app.
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ComputePipeline<S>>()
            .init_resource::<ComputeNodeState<S>>()
            .add_systems(
                ExtractSchedule,
                ComputeNode::<S>::reset_on_change
                    .run_if(resource_exists_and_changed::<S>)
                    .after(extract_resource::<S>),
            )
            .add_systems(
                ExtractSchedule,
                ComputeNodeState::<S>::extract_to_main
                    .run_if(resource_changed::<ComputeNodeState<S>>),
            )
            .add_systems(
                Render,
                S::prepare_bind_group
                    .in_set(RenderSystems::PrepareBindGroups)
                    .run_if(
                        not(resource_exists::<ComputeShaderBindGroup<S>>).or(resource_changed::<S>),
                    ),
            );

        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(
            ComputeNodeLabel::<S>::default(),
            ComputeNode::<S> {
                ..Default::default()
            },
        );
        render_graph.add_node_edge(
            ComputeNodeLabel::<S>::default(),
            bevy::render::graph::CameraDriverLabel,
        );
    }
}

/// Trait to implement for a custom compute shader.
pub trait ComputeShader: AsBindGroup + Clone + Debug + FromWorld + ExtractResource {
    /// Get a unique name for this class.
    fn unique_name() -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Asset path or handle to the shader.
    fn compute_shader() -> ShaderRef;
    /// Workgroup size. Must be the same across all instances.
    fn workgroup_size() -> UVec3;
    /// Workgroup count.
    fn workgroup_count(&self) -> UVec3;
    /// Alpha mode.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    /// Optional bind group preparation.
    fn prepare_bind_group(
        mut commands: Commands,
        pipeline: Res<ComputePipeline<Self>>,
        pipeline_cache: Res<PipelineCache>,
        render_device: Res<RenderDevice>,
        input: Res<Self>,
        param: StaticSystemParam<<Self as AsBindGroup>::Param>,
    ) {
        let bind_group = input
            .as_bind_group(
                &pipeline.resources.layout,
                &render_device,
                &pipeline_cache,
                &mut param.into_inner(),
            )
            .unwrap();
        commands.insert_resource(ComputeShaderBindGroup::<Self> {
            bind_group: bind_group.bind_group,
            _marker: PhantomData,
        });
    }
}

/// Stores prepared bind group data for the compute shader.
#[derive(Resource)]
pub struct ComputeShaderBindGroup<S: ComputeShader> {
    pub bind_group: BindGroup,
    pub _marker: PhantomData<S>,
}

/// Enum representing possible compute node states.
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ComputeNodeStatus {
    #[default]
    Loading,
    Update,
    Error,
}
/// Tracks compute node state.
/// In render world, this is stored as a resource which is later extracted to main.
/// In main world, this is a state so systems can react to state entry.
#[derive(States, Resource, Clone, Copy, Debug)]
pub struct ComputeNodeState<S: ComputeShader> {
    status: ComputeNodeStatus,
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> Hash for ComputeNodeState<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.status.hash(state);
    }
}
impl<S: ComputeShader> PartialEq for ComputeNodeState<S> {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
    }
}
impl<S: ComputeShader> Eq for ComputeNodeState<S> {}
impl<S: ComputeShader> From<ComputeNodeStatus> for ComputeNodeState<S> {
    fn from(value: ComputeNodeStatus) -> Self {
        Self {
            status: value,
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> Default for ComputeNodeState<S> {
    fn default() -> Self {
        Self {
            status: ComputeNodeStatus::default(),
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> ComputeNodeState<S> {
    /// Extracts compute node state resource into a state
    /// that systems can react to in the main world.
    fn extract_to_main(compute_state: Res<ComputeNodeState<S>>, mut world: ResMut<MainWorld>) {
        world
            .resource_mut::<NextState<ComputeNodeState<S>>>()
            .set(compute_state.clone());
    }
}

struct ComputePipelineResources<S: ComputeShader> {
    pub layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> ComputePipelineResources<S> {
    pub fn new(
        shader: Handle<Shader>,
        workgroup_size: UVec3,
        layout: BindGroupLayoutDescriptor,
        pipeline_cache: &PipelineCache,
    ) -> Self {
        let shader_defs = vec![
            ShaderDefVal::UInt("WORKGROUP_SIZE_X".into(), workgroup_size.x),
            ShaderDefVal::UInt("WORKGROUP_SIZE_Y".into(), workgroup_size.y),
            ShaderDefVal::UInt("WORKGROUP_SIZE_Z".into(), workgroup_size.z),
        ];
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(S::unique_name().into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: shader_defs.clone(),
            entry_point: Some("update".into()),
            zero_initialize_workgroup_memory: true,
        });
        Self {
            layout,
            pipeline,
            _marker: PhantomData,
        }
    }
}

/// Defines the pipeline for the compute shader.
#[derive(Resource)]
pub struct ComputePipeline<S: ComputeShader> {
    resources: ComputePipelineResources<S>,
}
impl<S: ComputeShader> FromWorld for ComputePipeline<S> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let shader_ref = match S::compute_shader() {
            ShaderRef::Default => panic!("Must define compute_shader."),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => world.load_asset(path),
        };
        Self {
            resources: ComputePipelineResources::<S>::new(
                shader_ref,
                S::workgroup_size(),
                S::bind_group_layout_descriptor(render_device),
                world.resource::<PipelineCache>(),
            ),
        }
    }
}

/// Label to identify the node in the render graph.
#[derive(Debug, Clone, RenderLabel)]
struct ComputeNodeLabel<S: ComputeShader> {
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> Default for ComputeNodeLabel<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> PartialEq for ComputeNodeLabel<S> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<S: ComputeShader> Eq for ComputeNodeLabel<S> {}
impl<S: ComputeShader> Hash for ComputeNodeLabel<S> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

/// The node that will execute the compute shader.
/// Updates `ComputeNodeState<S>` in the `RenderWorld`.
struct ComputeNode<S: ComputeShader> {
    status: ComputeNodeStatus,
    _marker: PhantomData<S>,
}
impl<S: ComputeShader> Default for ComputeNode<S> {
    fn default() -> Self {
        Self {
            status: ComputeNodeStatus::default(),
            _marker: PhantomData,
        }
    }
}
impl<S: ComputeShader> ComputeNode<S> {
    /// When the input shader is changed, reset.
    fn reset_on_change(
        mut render_graph: ResMut<RenderGraph>,
        mut state: ResMut<ComputeNodeState<S>>,
    ) {
        let Ok(node) = render_graph.get_node_mut::<Self>(ComputeNodeLabel::<S>::default()) else {
            return;
        };
        node.status = ComputeNodeStatus::Loading;
        *state = ComputeNodeState {
            status: ComputeNodeStatus::Loading,
            ..Default::default()
        };
    }
    /// Check pipeline load state.
    fn next_state(
        &self,
        pipeline: &ComputePipeline<S>,
        pipeline_cache: &PipelineCache,
    ) -> ComputeNodeStatus {
        let pipeline_state = pipeline_cache.get_compute_pipeline_state(pipeline.resources.pipeline);
        match pipeline_state {
            CachedPipelineState::Creating(_) | CachedPipelineState::Queued => {
                return ComputeNodeStatus::Loading;
            }
            CachedPipelineState::Err(_) => {
                return ComputeNodeStatus::Error;
            }
            _ => {}
        }
        match self.status {
            ComputeNodeStatus::Loading => ComputeNodeStatus::Update,
            ComputeNodeStatus::Update => ComputeNodeStatus::Update,
            _ => ComputeNodeStatus::Error,
        }
    }
}
impl<S: ComputeShader> render_graph::Node for ComputeNode<S> {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<ComputePipeline<S>>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let next_status = self.next_state(pipeline, pipeline_cache);
        if self.status != next_status {
            self.status = next_status;
            world.resource_mut::<ComputeNodeState<S>>().status = next_status;
        }
    }
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let shader = world.resource::<S>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline<S>>();
        let bind_group = &world.resource::<ComputeShaderBindGroup<S>>().bind_group;
        match self.status {
            ComputeNodeStatus::Update => {
                if let Some(pipeline) =
                    pipeline_cache.get_compute_pipeline(pipeline.resources.pipeline)
                {
                    let workgroup_count = shader.workgroup_count();
                    let mut pass = render_context.command_encoder().begin_compute_pass(
                        &ComputePassDescriptor {
                            label: Some(S::unique_name()),
                            ..Default::default()
                        },
                    );
                    pass.set_bind_group(0, bind_group, &[]);
                    pass.set_pipeline(pipeline);
                    pass.dispatch_workgroups(
                        workgroup_count.x,
                        workgroup_count.y,
                        workgroup_count.z,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
}

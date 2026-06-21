use bevy::{
    app::{Plugin, Update},
    asset::{Assets, DirectAssetAccessExt, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        lifecycle::HookContext,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Query, ResMut},
        world::{DeferredWorld, FromWorld, World},
    },
    math::{Vec2, Vec3Swizzles},
    mesh::MeshTag,
    reflect::Reflect,
    render::{
        extract_resource::ExtractResource, render_resource::ShaderType, storage::ShaderBuffer,
    },
    transform::{
        TransformSystems,
        components::{GlobalTransform, Transform},
    },
};

use crate::{SoftBody2dVertexBuffer, compute::SoftBodyCompute, instances::SoftBody2dInstanceData};

#[derive(Default)]
pub struct SoftBodyNodesPlugin<const N: usize>;
impl<const N: usize> Plugin for SoftBodyNodesPlugin<N> {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<SoftBody2dNodeDataBuffer<N>>()
            .add_systems(
                Update,
                (
                    SoftBodyNodes::<N>::update,
                    SoftBodyNodes::<N>::update_buffers,
                )
                    .after(TransformSystems::Propagate)
                    .chain(),
            );
    }
}

#[derive(Default, ShaderType, Copy, Clone, Debug)]
pub struct SoftBody2dNodeData {
    pub position: Vec2,
    pub radius: f32,
}

#[derive(Resource, Clone, ExtractResource)]
pub struct SoftBody2dNodeDataBuffer<const N: usize>(pub Handle<ShaderBuffer>);
impl<const N: usize> FromWorld for SoftBody2dNodeDataBuffer<N> {
    fn from_world(world: &mut World) -> Self {
        Self(world.add_asset(ShaderBuffer::from(Vec::<SoftBody2dNodeData>::new())))
    }
}

#[derive(Component, Reflect)]
pub struct SoftBody2dNode {
    pub radius: f32,
}

pub const MAX_NODES: usize = 1024;
pub const MAX_INSTANCES: usize = 256;

#[derive(Component, Reflect)]
#[component(on_add = SoftBodyNodes::<N>::on_add, on_remove = SoftBodyNodes::<N>::on_remove)]
pub struct SoftBodyNodes<const N: usize>(pub Vec<Entity>);
impl<const N: usize> SoftBodyNodes<N> {
    /// Initialize soft body on component add.
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let num_instances = world.resource_mut::<SoftBodyCompute<N>>().add_instance();
        world
            .commands()
            .entity(context.entity)
            .insert(MeshTag(num_instances - 1));

        let buffer_handle = world.resource_mut::<SoftBody2dVertexBuffer<N>>().0.clone();
        if let Some(mut buffer) = world
            .resource_mut::<Assets<ShaderBuffer>>()
            .get_mut(&buffer_handle)
        {
            SoftBody2dVertexBuffer::<N>::resize_buffer(num_instances, &mut buffer);
        }
    }

    /// Decrement compute counter on component remove.
    fn on_remove(mut world: DeferredWorld, _context: HookContext) {
        let num_instances = world.resource_mut::<SoftBodyCompute<N>>().remove_instance();
        let buffer_handle = world.resource_mut::<SoftBody2dVertexBuffer<N>>().0.clone();
        if let Some(mut buffer) = world
            .resource_mut::<Assets<ShaderBuffer>>()
            .get_mut(&buffer_handle)
        {
            SoftBody2dVertexBuffer::<N>::resize_buffer(num_instances, &mut buffer);
        }
    }

    /// Update to the center of mass of all nodes.
    pub fn update(
        mut query: Query<(&mut Transform, &Self)>,
        node_transforms: Query<&GlobalTransform, With<SoftBody2dNode>>,
    ) {
        for (mut transform, nodes) in query.iter_mut() {
            let mut sum_pos = Vec2::ZERO;
            for entity in &nodes.0 {
                if let Ok(node_transform) = node_transforms.get(*entity) {
                    sum_pos += node_transform.translation().xy();
                }
            }
            let centroid = sum_pos / (nodes.0.len() as f32);
            transform.translation.x = centroid.x;
            transform.translation.y = centroid.y;
        }
    }

    /// Copy relative positions into the nodes buffer.
    pub fn update_buffers(
        mut compute: ResMut<SoftBodyCompute<N>>,
        mut buffers: ResMut<Assets<ShaderBuffer>>,
        query: Query<(&GlobalTransform, &SoftBodyNodes<N>)>,
        node_transforms: Query<(&GlobalTransform, &SoftBody2dNode)>,
    ) {
        let mut all_nodes = [SoftBody2dNodeData::default(); MAX_NODES];
        let mut all_instances = [SoftBody2dInstanceData::default(); MAX_INSTANCES];

        let mut node_i = 0;
        let mut instance_i = 0;

        for (transform, nodes) in query.iter() {
            let node_offset = node_i;
            for entity in &nodes.0 {
                if let Ok((node_transform, node)) = node_transforms.get(*entity) {
                    let rel_transform = node_transform.reparented_to(transform);
                    all_nodes[node_i] = SoftBody2dNodeData {
                        position: rel_transform.translation.xy(),
                        radius: node.radius,
                    };
                    node_i += 1;
                }
            }
            all_instances[instance_i] = SoftBody2dInstanceData {
                node_offset: node_offset as u32,
                node_length: (node_i - node_offset) as u32,
            };
            instance_i += 1;
        }
        compute.num_instances = instance_i as u32;
        if let Some(mut node_buffer) = buffers.get_mut(&compute.nodes) {
            node_buffer.set_data(&all_nodes[0..node_i]);
        }
        if let Some(mut instance_buffer) = buffers.get_mut(&compute.instances) {
            instance_buffer.set_data(&all_instances[0..instance_i]);
        }
    }
}

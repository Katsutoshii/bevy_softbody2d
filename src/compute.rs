use bevy::{
    app::{App, Plugin},
    asset::{Asset, Handle},
    ecs::{
        resource::Resource,
        world::{FromWorld, World},
    },
    math::UVec3,
    reflect::{Reflect, TypePath},
    render::{
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
};

use crate::{
    COMPUTE_SHADER_ASSET_PATH, ComputeShader, ComputeShaderPlugin, SoftBody2dVertexBuffer,
    instances::SoftBody2dInstanceBuffer, nodes::SoftBody2dNodeDataBuffer,
};

#[derive(Default)]
pub struct SoftBodyComputePlugin<const N: usize>;
impl<const N: usize> Plugin for SoftBodyComputePlugin<N> {
    fn build(&self, app: &mut App) {
        app.add_plugins((ComputeShaderPlugin::<SoftBodyCompute<N>>::default(),));
    }
}
#[derive(ShaderType, Reflect, Clone, Debug)]
pub struct SoftBodyComputeUniform {
    pub smooth_steps: u32,
}
impl Default for SoftBodyComputeUniform {
    fn default() -> Self {
        Self { smooth_steps: 2 }
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Resource, ExtractResource)]
pub struct SoftBodyCompute<const N: usize> {
    #[uniform(0)]
    num_vertices_per_instance: u32,
    #[uniform(1)]
    uniforms: SoftBodyComputeUniform,

    #[storage(2, visibility(compute))]
    pub vertices: Handle<ShaderStorageBuffer>,
    #[storage(3, read_only, visibility(compute))]
    pub nodes: Handle<ShaderStorageBuffer>,
    #[storage(4, read_only, visibility(compute))]
    pub instances: Handle<ShaderStorageBuffer>,

    // Track number of instances for dispatching workgroups.
    pub num_instances: u32,
}
impl<const N: usize> FromWorld for SoftBodyCompute<N> {
    fn from_world(world: &mut World) -> Self {
        Self {
            num_vertices_per_instance: N as u32,
            uniforms: SoftBodyComputeUniform { smooth_steps: 2 },
            vertices: world.resource::<SoftBody2dVertexBuffer<N>>().0.clone(),
            instances: world.resource::<SoftBody2dInstanceBuffer<N>>().0.clone(),
            nodes: world.resource::<SoftBody2dNodeDataBuffer<N>>().0.clone(),
            num_instances: 0,
        }
    }
}
impl<const N: usize> SoftBodyCompute<N> {
    pub fn add_instance(&mut self) -> u32 {
        self.num_instances += 1;
        self.num_instances
    }

    pub fn remove_instance(&mut self) -> u32 {
        self.num_instances -= 1;
        self.num_instances
    }
}
impl<const N: usize> ComputeShader for SoftBodyCompute<N> {
    fn compute_shader() -> ShaderRef {
        ShaderRef::Path(COMPUTE_SHADER_ASSET_PATH.clone())
    }
    fn workgroup_size() -> UVec3 {
        UVec3::new(N as u32, 1, 1)
    }
    fn workgroup_count(&self) -> UVec3 {
        UVec3::new(1, self.num_instances, 1)
    }
}

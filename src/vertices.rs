use bevy::{
    app::Plugin,
    asset::{DirectAssetAccessExt, Handle},
    ecs::{
        resource::Resource,
        world::{FromWorld, World},
    },
    math::Vec2,
    render::{
        extract_resource::ExtractResource, render_resource::ShaderType,
        storage::ShaderStorageBuffer,
    },
};

#[derive(Default)]
pub struct SoftBodyVerticesPlugin<const N: usize>;
impl<const N: usize> Plugin for SoftBodyVerticesPlugin<N> {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<SoftBody2dVertexBuffer<N>>();
    }
}

#[derive(Default, ShaderType, Copy, Clone, Debug)]
pub struct SoftBody2dVertex {
    pub position: Vec2,
}

#[derive(Resource, ExtractResource, Clone)]
pub struct SoftBody2dVertexBuffer<const N: usize>(pub Handle<ShaderStorageBuffer>);
impl<const N: usize> SoftBody2dVertexBuffer<N> {
    /// Resizes the vertex buffer to support the given number of instances.
    pub fn resize_buffer(num_instances: u32, buffer: &mut ShaderStorageBuffer) {
        let element_size = SoftBody2dVertex::min_size().get() as usize;
        let new_buffer_size = element_size * num_instances as usize * N;
        if let Some(data) = buffer.data.as_mut() {
            data.resize(new_buffer_size, 0u8);
        }
    }
}
impl<const N: usize> FromWorld for SoftBody2dVertexBuffer<N> {
    fn from_world(world: &mut World) -> Self {
        Self(world.add_asset(ShaderStorageBuffer::from(Vec::<SoftBody2dVertex>::new())))
    }
}

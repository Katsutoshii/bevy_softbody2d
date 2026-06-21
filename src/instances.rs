use bevy::{
    app::Plugin,
    asset::{DirectAssetAccessExt, Handle},
    ecs::{
        resource::Resource,
        world::{FromWorld, World},
    },
    render::{
        extract_resource::ExtractResource, render_resource::ShaderType, storage::ShaderBuffer,
    },
};

#[derive(Default)]
pub struct SoftBodyInstancesPlugin<const N: usize>;
impl<const N: usize> Plugin for SoftBodyInstancesPlugin<N> {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<SoftBody2dInstanceBuffer<N>>();
    }
}

#[derive(Default, ShaderType, Copy, Clone, Debug)]
pub struct SoftBody2dInstanceData {
    pub node_offset: u32,
    pub node_length: u32,
}

#[derive(Resource, ExtractResource, Clone)]
pub struct SoftBody2dInstanceBuffer<const N: usize>(pub Handle<ShaderBuffer>);
impl<const N: usize> FromWorld for SoftBody2dInstanceBuffer<N> {
    fn from_world(world: &mut World) -> Self {
        Self(world.add_asset(ShaderBuffer::from(Vec::<SoftBody2dInstanceData>::new())))
    }
}

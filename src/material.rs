use bevy::{
    app::{App, Plugin},
    asset::{Asset, DirectAssetAccessExt, Handle},
    color::LinearRgba,
    ecs::{
        lifecycle::HookContext,
        resource::Resource,
        world::{DeferredWorld, FromWorld, World},
    },
    image::Image,
    material::AlphaMode,
    mesh::{Mesh, Mesh3d, MeshVertexBufferLayoutRef},
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin, MeshMaterial3d},
    reflect::{Reflect, TypePath},
    render::{
        extract_resource::ExtractResource,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
        },
        storage::ShaderBuffer,
    },
    shader::ShaderRef,
};

use crate::{
    CircleNGon, MATERIAL_SHADER_ASSET_PATH, SoftBody2dVertexBuffer, compute::SoftBodyComputePlugin,
    instances::SoftBodyInstancesPlugin, nodes::SoftBodyNodesPlugin,
    vertices::SoftBodyVerticesPlugin,
};

#[derive(Default)]
pub struct SoftBodyMaterial2dPlugin<const N: usize>;
impl<const N: usize> Plugin for SoftBodyMaterial2dPlugin<N> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<SoftBody2dMaterial<N>>::default(),
            SoftBodyVerticesPlugin::<N>::default(),
            SoftBodyInstancesPlugin::<N>::default(),
            SoftBodyNodesPlugin::<N>::default(),
            SoftBodyComputePlugin::<N>::default(),
        ))
        .init_resource::<SoftBody2dMeshHandle<N>>();

        app.world_mut()
            .register_component_hooks::<MeshMaterial3d<SoftBody2dMaterial<N>>>()
            .on_add(SoftBody2dMaterial::<N>::on_add);
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Resource, ExtractResource)]
pub struct SoftBody2dMaterial<const N: usize> {
    #[uniform(0)]
    pub num_vertices_per_instance: u32,
    #[uniform(1)]
    pub uniforms: SoftBodyMaterialUniform,

    #[texture(2)]
    #[sampler(3)]
    pub color_texture: Option<Handle<Image>>,

    #[storage(4, read_only)]
    pub vertices: Handle<ShaderBuffer>,

    pub alpha_mode: AlphaMode,
}
impl<const N: usize> FromWorld for SoftBody2dMaterial<N> {
    fn from_world(world: &mut World) -> Self {
        Self {
            num_vertices_per_instance: N as u32,
            uniforms: SoftBodyMaterialUniform::default(),
            color_texture: None,
            vertices: world.resource::<SoftBody2dVertexBuffer<N>>().0.clone(),
            alpha_mode: AlphaMode::default(),
        }
    }
}
impl<const N: usize> SoftBody2dMaterial<N> {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let mesh = world.resource::<SoftBody2dMeshHandle<N>>().0.clone();
        world.commands().entity(context.entity).insert(Mesh3d(mesh));
    }
}
impl<const N: usize> Material for SoftBody2dMaterial<N> {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(MATERIAL_SHADER_ASSET_PATH.clone())
    }
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(MATERIAL_SHADER_ASSET_PATH.clone())
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            CircleNGon::SOFT_BODY_INDEX_VERTEX_ATTRIBUTE.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

#[derive(ShaderType, Clone, Debug, Reflect, Default)]
pub struct SoftBodyMaterialUniform {
    pub color: LinearRgba,
}

#[derive(Resource)]
pub struct SoftBody2dMeshHandle<const N: usize>(pub Handle<Mesh>);
impl<const N: usize> FromWorld for SoftBody2dMeshHandle<N> {
    fn from_world(world: &mut World) -> Self {
        Self(world.add_asset(CircleNGon {
            n: (N - 1) as usize,
            r: 1.0,
        }))
    }
}

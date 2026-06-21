use bevy::{
    app::{App, Plugin},
    asset::{AssetPath, embedded_asset, embedded_path},
    shader::load_shader_library,
};

mod compute;
mod compute_shader;
mod instances;
mod material;
mod mesh;
mod nodes;
mod vertices;

use std::sync::LazyLock;

pub use crate::{
    compute_shader::{ComputeShader, ComputeShaderPlugin},
    material::{SoftBody2dMaterial, SoftBodyMaterial2dPlugin, SoftBodyMaterialUniform},
    mesh::CircleNGon,
    nodes::{SoftBody2dNode, SoftBody2dNodeDataBuffer, SoftBodyNodes},
    vertices::{SoftBody2dVertex, SoftBody2dVertexBuffer},
};

static MATERIAL_SHADER_ASSET_PATH: LazyLock<AssetPath> = LazyLock::new(|| {
    AssetPath::from_path_buf(embedded_path!("material.wgsl")).with_source("embedded")
});
static COMPUTE_SHADER_ASSET_PATH: LazyLock<AssetPath> = LazyLock::new(|| {
    AssetPath::from_path_buf(embedded_path!("compute.wgsl")).with_source("embedded")
});

pub struct SoftBodyPlugin;
impl Plugin for SoftBodyPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "material.wgsl");
        embedded_asset!(app, "compute.wgsl");
        load_shader_library!(app, "nodes.wgsl");
        load_shader_library!(app, "instances.wgsl");
        load_shader_library!(app, "rand.wgsl");
        load_shader_library!(app, "vertices.wgsl");
    }
}

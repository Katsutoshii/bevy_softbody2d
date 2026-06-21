use std::f32::consts::TAU;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{
        Indices, Mesh, MeshBuilder, MeshVertexAttribute, Meshable, PrimitiveTopology, VertexFormat,
    },
};

#[derive(Clone, Copy)]
pub struct CircleNGon {
    pub n: usize,
    pub r: f32,
}
impl CircleNGon {
    pub const SOFT_BODY_INDEX_VERTEX_ATTRIBUTE: MeshVertexAttribute = MeshVertexAttribute::new(
        "SoftBodyIndex",
        (0x7512b1e2bb023882) as u64,
        VertexFormat::Uint32,
    );
}
impl Meshable for CircleNGon {
    type Output = CircleNGonMeshBuilder;
    fn mesh(&self) -> Self::Output {
        CircleNGonMeshBuilder(*self)
    }
}

pub struct CircleNGonMeshBuilder(pub CircleNGon);
impl MeshBuilder for CircleNGonMeshBuilder {
    fn build(&self) -> Mesh {
        let CircleNGon { n, r } = self.0;

        let mut positions = Vec::with_capacity(n + 1);
        let mut uvs = Vec::with_capacity(n + 1);
        let mut indices = Vec::<u32>::with_capacity(n * 3);

        // Center vertex (Index 0)
        positions.push([0.0, 0.0, 0.0]);
        uvs.push([0.5, 0.5]); // Center of UV texture space

        // Perimeter vertices (Indices 1 to N)
        for i in 0..n {
            let angle = (i as f32 / n as f32) * TAU;

            let x = r * angle.cos();
            let y = r * angle.sin();

            positions.push([x, y, 0.0]);

            // Map [-radius, radius] coordinates to a normalized [0.0, 1.0] UV range
            let u = (x / r) * 0.5 + 0.5;
            let v = (-y / r) * 0.5 + 0.5;
            uvs.push([u, v]);
        }

        // Build the N-gon fan by linking the center vertex (0) to outer pairs
        for i in 1..n {
            indices.extend([0, i as u32, (i + 1) as u32]);
        }

        // Final triangle: connects the last vertex back to the first perimeter vertex (1)
        indices.extend([0, n as u32, 1]);

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(
            CircleNGon::SOFT_BODY_INDEX_VERTEX_ATTRIBUTE,
            (0..=n as u32).collect::<Vec<u32>>(),
        )
        .with_computed_smooth_normals()
    }
}

impl Into<Mesh> for CircleNGon {
    fn into(self) -> Mesh {
        self.mesh().build()
    }
}

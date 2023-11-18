use bevy :: prelude :: *;
use bevy :: render :: render_resource :: PrimitiveTopology;
use bevy :: render :: mesh :: { Indices, Mesh };

use lyon :: tessellation :: *;

pub type ABGlyphPoint   = ab_glyph :: Point;
pub type LyonPoint      = lyon :: math :: Point;

pub type VertexPos		= [f32; 3];
pub type VertexIndex	= u16;
pub type VertexUV		= [f32; 2];
pub type VertexBuffer	= VertexBuffers<VertexPos, VertexIndex>;
pub type NormalBuffer	= Vec<VertexPos>;
pub type UVBuffer		= Vec<VertexUV>;

#[derive(Default, Clone)]
pub struct MeshInternal {
	pub vertex_buffer	: VertexBuffer,
	pub normals			: NormalBuffer,
	pub uvs				: Option<UVBuffer>
}

impl MeshInternal {
	pub fn vertices(&self) -> &Vec<VertexPos> {
		&self.vertex_buffer.vertices
	}

	pub fn vertices_mut(&mut self) -> &mut Vec<VertexPos> {
		&mut self.vertex_buffer.vertices
	}

	pub fn indices(&self) -> &Vec<VertexIndex> {
		&self.vertex_buffer.indices
	}

	pub fn indices_mut(&mut self) -> &mut Vec<VertexIndex> {
		&mut self.vertex_buffer.indices
	}
}

pub fn generate_quad_vertices(
	mesh			: &mut MeshInternal,
	scale			: f32,
) {
	let vertices_cnt = 4;
	let v0			= Vec3::new(0.0, 0.0, 0.0) * scale;
	let v1			= Vec3::new(0.0, 1.0, 0.0) * scale;
	let v2			= Vec3::new(1.0, 1.0, 0.0) * scale;
	let v3			= Vec3::new(1.0, 0.0, 0.0) * scale;

	mesh.vertex_buffer.vertices.reserve(vertices_cnt);

	mesh.vertex_buffer.vertices.push(v0.to_array());
	mesh.vertex_buffer.vertices.push(v1.to_array());
	mesh.vertex_buffer.vertices.push(v2.to_array());
	mesh.vertex_buffer.vertices.push(v3.to_array());

	mesh.vertex_buffer.indices.reserve(vertices_cnt);

	mesh.vertex_buffer.indices.push(0);
	mesh.vertex_buffer.indices.push(2);
	mesh.vertex_buffer.indices.push(1);

	mesh.vertex_buffer.indices.push(0);
	mesh.vertex_buffer.indices.push(3);
	mesh.vertex_buffer.indices.push(2);

	mesh.normals 	= vec![[0.0, 0.0, 1.0]; vertices_cnt];

	let mut uvs		= Vec::with_capacity(vertices_cnt);

	let uv0			= Vec2::new(0.0, 1.0);
	let uv1			= Vec2::new(0.0, 0.0);
	let uv2			= Vec2::new(1.0, 0.0);
	let uv3			= Vec2::new(1.0, 1.0);

	uvs.push		(uv0.to_array());
	uvs.push		(uv1.to_array());
	uvs.push		(uv2.to_array());
	uvs.push		(uv3.to_array());
	mesh.uvs		= Some(uvs);
}

pub fn bevy_mesh_from_internal(
	mesh_internal		: &MeshInternal
) -> Mesh {
	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
	mesh.insert_attribute	(Mesh::ATTRIBUTE_POSITION, mesh_internal.vertices().clone());
	mesh.insert_attribute	(Mesh::ATTRIBUTE_NORMAL, mesh_internal.normals.clone());
	mesh.set_indices(Some	(Indices::U16(mesh_internal.indices().clone())));
	if let Some(uvs) = &mesh_internal.uvs {
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone())
	}

	mesh
}
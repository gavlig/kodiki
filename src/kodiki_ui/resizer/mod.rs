use bevy :: prelude :: *;
use bevy :: render :: render_resource :: PrimitiveTopology;
use bevy :: render :: mesh :: { Indices, Mesh };

use bevy_rapier3d :: prelude :: *;

use crate :: z_order;

use super :: raypick :: RaypickHover;

pub mod systems;

#[derive(Component)]
pub struct ResizerHighlight;

#[derive(Component)]
pub struct Resizer {
	pub width				: f32,
	pub height				: f32,
	pub margin				: f32,
	pub quad_color			: Color,
		quad_color_cached	: Color,
	pub circles_color		: Color,

	pub init_mouse_pos		: Option<Vec2>,
	pub init_area			: Option<UVec2>,
	pub area				: UVec2, // x = columns, y = rows. TODO: use a dedicated struct

	pub entity				: Entity,
	pub quad_entity			: Entity,
	pub circles_entity		: Entity,
	pub surface_name		: String,
}

const RESIZER_WIDTH			: f32 = 0.1;
const RESIZER_HEIGHT		: f32 = 2.0;
const RESIZER_MARGIN		: f32 = RESIZER_WIDTH / 2.0;

impl Default for Resizer {
	fn default() -> Self {
		Self {
			width			: RESIZER_WIDTH,
			height			: RESIZER_HEIGHT,
			margin			: RESIZER_MARGIN,
			quad_color		: Color::CYAN,
			quad_color_cached : Color::CYAN,
			circles_color	: Color::CYAN,

			init_mouse_pos	: None,
			init_area		: None,
			area			: UVec2::ZERO,

			entity			: Entity::from_raw(0),
			quad_entity		: Entity::from_raw(0),
			circles_entity	: Entity::from_raw(0),

			surface_name	: String::new(),
		}
	}
}

impl Resizer {
	pub fn spawn(
		surface_name	: &str,
		area			: UVec2,
		mesh_assets		: &mut Assets<Mesh>,
		material_assets	: &mut Assets<StandardMaterial>,
		commands		: &mut Commands
	) -> Entity {
		let quad_size	= Vec2::new(Resizer::default().width, Resizer::default().height);

		let quad_mesh_handle = mesh_assets.add(shape::Quad::new(quad_size).into());
		let quad_material_handle = material_assets.add(StandardMaterial {
			base_color: Resizer::default().quad_color,
			unlit : true,
			..default()
		});

		let circles_mesh_handle = mesh_assets.add(mesh_circles(shape::Quad::new(quad_size)));
		let circles_material_handle = material_assets.add(StandardMaterial {
			base_color: Resizer::default().quad_color,
			unlit : true,
			..default()
		});

		let quad_entity = commands.spawn((
			PbrBundle {
				mesh		: quad_mesh_handle,
				material	: quad_material_handle,
				transform	: Transform::from_translation(Vec3::Z * z_order::resizer()),
				..default()
			},
		)).id();

		let circles_entity = commands.spawn((
			PbrBundle {
				mesh		: circles_mesh_handle,
				material	: circles_material_handle,
				transform	: Transform::from_translation(Vec3::Z * z_order::resizer()),
				..default()
			},
		)).id();

		commands.entity(quad_entity).add_child(circles_entity);

		let resizer_entity = commands.spawn((
			TransformBundle::default(),
			VisibilityBundle::default(),
		)).id();

		let resizer = Resizer {
			entity: resizer_entity,
			quad_entity,
			circles_entity,
			area,
			surface_name: String::from(surface_name),
			..default()
		};

		commands.entity(resizer_entity)
			.insert((
			resizer,
			RigidBody	:: Fixed,
			Collider	:: cuboid(quad_size.x / 2., quad_size.y / 2., z_order::thickness() / 2.),
			RaypickHover:: default()
			))
			.push_children(&[quad_entity])
		;

		resizer_entity
	}

	pub fn dragging_active(&self) -> bool {
		self.init_mouse_pos.is_some()
	}

}

fn mesh_circles(quad: shape::Quad) -> Mesh {
	let num_circles = 8;
	let circles_per_row = 2;
	let circles_per_column = num_circles / circles_per_row;

	let circle_resolution = 12;

	let diameter_full = quad.size.x / circles_per_row as f32;
	let circle_radius_full = diameter_full / 2.0;
	let circle_margin = circle_radius_full / 3.0;
	let circle_radius_inner = circle_radius_full - circle_margin;

	let num_vertices_circle = circle_resolution;
	let num_indices_circle = (circle_resolution - 2) * 3;

	let num_vertices = num_vertices_circle * num_circles;
	let num_indices = num_indices_circle * num_circles;

	let mut positions = Vec::with_capacity(num_vertices as usize);
	let mut normals = Vec::with_capacity(num_vertices as usize);
	let mut uvs = Vec::with_capacity(num_vertices as usize);
	let mut indices = Vec::with_capacity(num_indices as usize);

	let step_theta = std::f32::consts::TAU / circle_resolution as f32;

	let mut y_offset = 0.0;
	for circle_index in 0 .. num_circles {
		let offset = positions.len() as u32;

		let x_offset = (circle_index % circles_per_row) as f32 * diameter_full;
		if circle_index % circles_per_row == 0 && circle_index != 0 {
			y_offset += diameter_full;
		}

		for i in 0 .. circle_resolution {
			let theta = i as f32 * step_theta;
			let (sin, cos) = theta.sin_cos();

			let x = x_offset + cos * circle_radius_inner - (circle_radius_full * (circles_per_row - 1) as f32);
			let y = y_offset + sin * circle_radius_inner - (circle_radius_full * (circles_per_column - 1) as f32);

			let z = z_order::surface::text();
			positions.push([x, y, z]);
			normals.push([0.0, 0.0, 1.0]);
			uvs.push([0.5 * (cos + 1.0), 1.0 - 0.5 * (sin + 1.0)]);
		}

		for i in 1..(circle_resolution - 1) {
			indices.extend_from_slice(&[
				offset,
				offset + i ,
				offset + i + 1,
			]);
		}
	}

	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
	mesh.set_indices(Some(Indices::U32(indices)));
	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
	mesh
}
use bevy				:: prelude :: { * };
use bevy_mod_picking	:: { * };

use crate :: bevy_ab_glyph :: TextMeshesCache;

fn get_quad_mesh_handle(
	quad_mesh_name	: &String,
	quad_size		: Vec2,
	text_mesh_cache	: &mut TextMeshesCache,
	mesh_assets		: &mut Assets<Mesh>,
) -> Handle<Mesh> {
	let quad_mesh_handle = match text_mesh_cache.meshes.get(quad_mesh_name) {
		Some(handle) => handle.clone_weak(),
		None => {
			let handle = mesh_assets.add(Mesh::from(shape::Quad::new(quad_size)));
			
			text_mesh_cache.meshes.insert_unique_unchecked(quad_mesh_name.clone(), handle).1.clone()
		}
	};
	
	return quad_mesh_handle;
}

pub fn glyph_quad(
	quad_pos_in		: Vec3,
	quad_size		: Vec2,
	text_mesh_cache	: &mut TextMeshesCache,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) -> Entity {
	let quad_mesh_name	= String::from("glyph-background-quad");
	let quad_mesh_handle = get_quad_mesh_handle(&quad_mesh_name, quad_size, text_mesh_cache, mesh_assets);
	let quad_pos		= quad_pos_in + Vec3::new(quad_size.x / 2.0, -quad_size.y / 2.0, 0.0);

	commands.spawn(PbrBundle {
		mesh			: quad_mesh_handle.clone_weak(),
		transform		: Transform {
			translation	: quad_pos,
			// rotation	: Quat::from_rotation_y(std::f32::consts::PI), // winding ccw something something
			..default()
		},
		..default()
	})
	.id()
}

pub fn background_quad(
	quad_pos_in		: Vec3,
	quad_size		: Vec2,
	material_handle	: Option<&Handle<StandardMaterial>>,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) -> Entity {
	let quad_mesh_handle = mesh_assets.add(Mesh::from(shape::Quad::new(quad_size)));
	
	use bevy::ui::FocusPolicy;
	
	let entity = commands.spawn(PbrBundle {
		mesh			: quad_mesh_handle.clone(),
		transform		: Transform {
			translation	: quad_pos_in,
			..default()
		},
		..default()
	})
	
	// unpacked PickableBundle because we don't want Highlight
	.insert(PickableMesh::default())
	.insert(Interaction::default())
	.insert(FocusPolicy::default())
	.insert(Selection::default())
	.insert(Hover::default())
	
	.id();
	
	if let Some(handle) = material_handle {
		commands.entity(entity).insert(handle.clone_weak());
	}
	
	entity
}
use bevy :: prelude :: *;

use bevy_rapier3d :: prelude :: *;

use crate :: {
	z_order,
	kodiki_ui :: { self, *, color :: * },
	bevy_ab_glyph :: {
		ABGlyphFont, GlyphMeshesCache, TextMeshesCache,
		glyph_mesh_generator :: generate_string_mesh_wcache,
	}
};

pub fn background_quad(
	quad_pos		: Vec3,
	quad_size		: Vec2,
	with_collision	: bool,
	material_handle	: Option<&Handle<StandardMaterial>>,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) -> Entity {
	let quad_mesh_handle = mesh_assets.add(Mesh::from(Rectangle::from_size(quad_size)));

	let entity = commands.spawn(PbrBundle {
		mesh			: quad_mesh_handle.clone(),
		transform		: Transform {
			translation	: quad_pos,
			..default()
		},
		..default()
	})
	.id();

	if let Some(handle) = material_handle {
		commands.entity(entity).insert(handle.clone_weak());
	}

	if with_collision {
		commands.entity(entity).insert((
			RigidBody::Fixed,
			Collider::cuboid(quad_size.x / 2., quad_size.y / 2., z_order::thickness() / 2.),
		));
	}

	entity
}

pub fn mesh_material_entity(
	mesh_handle		: &Handle<Mesh>,
	material_handle	: &Handle<StandardMaterial>,
	commands		: &mut Commands
) -> Entity {
	commands.spawn(
		PbrBundle {
			mesh		: mesh_handle.clone_weak(),
			material	: material_handle.clone_weak(),
			..default()
		}
	)
	.id()
}

pub fn mesh_material_entity_wtranslation(
	mesh_handle		: &Handle<Mesh>,
	material_handle	: &Handle<StandardMaterial>,
	translation		: Vec3,
	commands		: &mut Commands
) -> Entity {
	let word_mesh_entity = commands.spawn(
		PbrBundle {
			mesh		: mesh_handle.clone_weak(),
			material	: material_handle.clone_weak(),
			transform	: Transform::from_translation(translation),
			..default()
		}
	)
	.id();

	word_mesh_entity
}

pub fn string_mesh_collision(
	word 			: &String,
	font			: &ABGlyphFont,
	commands		: &mut Commands
) -> Entity {
	let row_height	= font.vertical_advance();
	let column_width = font.horizontal_advance_mono();

	let width		= word.len();
	let height		= 1;

	let cube_width	= column_width * width as f32;
	let cube_height	= row_height * height as f32;

	let cube_x		= cube_width / 2.0;
	let cube_y		= cube_height / 2.0;

	let cube_pos	= Vec3::new(cube_x, cube_y, 0.0);

	commands.spawn((
		TransformBundle {
			local : Transform::from_translation(cube_pos),
			..default()
		},
		RigidBody::Fixed,
		Collider::cuboid(cube_width / 2., cube_height / 2., z_order::thickness() / 2.),
	)).id()
}

pub fn string_mesh(
	string		: &String,
	color		: Color,
	transform	: Transform,
	font		: &ABGlyphFont,

	mesh_assets					: &mut Assets<Mesh>,
	material_assets				: &mut Assets<StandardMaterial>,
	glyph_meshes_cache			: &mut GlyphMeshesCache,
	text_meshes_cache			: &mut TextMeshesCache,
	color_materials_cache		: &mut ColorMaterialsCache,

	commands					: &mut Commands
) -> Entity {
	let (string_mesh_handle, string_material_handle) = (
		generate_string_mesh_wcache(string, font, mesh_assets, glyph_meshes_cache, text_meshes_cache),
		get_color_material_handle(color, color_materials_cache, material_assets)
	);

	// spawning everything we need for word objects

	let string_mesh_entity = kodiki_ui::spawn::mesh_material_entity(
		&string_mesh_handle,
		&string_material_handle,
		commands
	);

	commands.entity(string_mesh_entity).insert(transform);

	string_mesh_entity
}

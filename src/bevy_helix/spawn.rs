use bevy				:: prelude :: { * };
use bevy_reader_camera	:: { * };
use bevy_contrib_colors	:: { Tailwind };

use super				:: { * };

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
use helix_view :: graphics :: Color as HelixColor;

use crate :: bevy_ab_glyph :: ABGlyphFont;
use crate :: bevy_ab_glyph :: TextMeshesCache;

pub fn quad(
	quad_pos_in		: Vec3,
	quad_size		: Vec2,
	text_mesh_cache	: &mut TextMeshesCache,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) -> Entity {
	let quad_mesh_name	= String::from("glyph-background-quad");
	let quad_width		= quad_size.x;
	let quad_height		= quad_size.y;

	let quad_mesh_handle = match text_mesh_cache.meshes.get(&quad_mesh_name) {
		Some(handle) => handle.clone_weak(),
		None => {
			let handle = mesh_assets.add(
				Mesh::from(
					shape::Quad::new(
						Vec2::new(
							quad_width,
							quad_height
						)
					)
				)
			);
			
			text_mesh_cache.meshes.insert_unique_unchecked(quad_mesh_name.clone(), handle).1.clone()
		}
	};

	let quad_pos		= quad_pos_in + Vec3::new(quad_width / 2.0, -quad_height / 2.0, 0.0);

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

fn color_from_helix(helix_color: HelixColor) -> Color {
	match helix_color {
		HelixColor::Reset => Color::WHITE,
		HelixColor::Black => Color::BLACK,
		HelixColor::Red => Tailwind::RED600,
		HelixColor::Green => Tailwind::GREEN600,
		HelixColor::Yellow => Tailwind::YELLOW600,
		HelixColor::Blue => Tailwind::BLUE600,
		HelixColor::Magenta => Tailwind::PURPLE600,
		HelixColor::Cyan => Color::rgb(0.0, 0.5, 0.5),
		HelixColor::Gray => Tailwind::GRAY600,
		HelixColor::LightRed => Tailwind::RED300,
		HelixColor::LightGreen => Tailwind::GREEN300,
		HelixColor::LightBlue => Tailwind::BLUE300,
		HelixColor::LightYellow => Tailwind::YELLOW300,
		HelixColor::LightMagenta => Tailwind::PURPLE300,
		HelixColor::LightCyan => Color::rgb(0.0, 0.7, 0.7),
		HelixColor::LightGray => Tailwind::GRAY300,
		HelixColor::White => Color::WHITE,
		// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
		HelixColor::Indexed(_i) => { panic!("Indexed color is not supported!"); },// Color::AnsiValue(i), 
		HelixColor::Rgb(r, g, b) => Color::rgb_u8(r, g, b),
	}
}

pub fn surface(
	surface_name	: &String,
	world_position	: Option<Vec3>,
	surfaces_bevy	: &mut SurfacesMapBevy,
	surface_helix	: &SurfaceHelix,
	font			: &ABGlyphFont,
	text_meshes_cache : &mut TextMeshesCache,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) -> Entity
{
	println!		("spawning surface {}", surface_name);
	
	let surface_position = world_position.unwrap_or(Vec3::new(0.0, 0.0, 0.0));
	let surface_entity =
	commands.spawn(TransformBundle {
		local		: Transform::from_translation(surface_position),
		..default()
	})
	.insert(VisibilityBundle {
		visibility	: Visibility { is_visible: true },
		..default()
	})
	.id();
	
	let mut surface_bevy = SurfaceBevy::new_with_entity(surface_entity);
	
	fill::surface(surface_name, &mut surface_bevy, surface_helix, font, text_meshes_cache, mesh_assets, commands);
	
	surfaces_bevy.insert(surface_name.clone(), surface_bevy);

	surface_entity
}

pub fn cursor(
	cursor			: &mut CursorBevy,
	
	surface_entity	: Entity,
	font			: &ABGlyphFont,

	text_meshes_cache : &mut TextMeshesCache,
	helix_colors_cache : &mut HelixColorsCache,

	material_assets	: &mut Assets<StandardMaterial>,
	mesh_assets		: &mut ResMut<Assets<Mesh>>,
	commands		: &mut Commands
)
{
	let cursor_color_fg	= color_from_helix(HelixColor::Magenta);
	let material_handle	= get_helix_color_material_handle(cursor_color_fg, helix_colors_cache, material_assets);
	
	let v_advance		= font.vertical_advance();
	let h_advance		= font.horizontal_advance(&String::from("a")); // in monospace font every letter should be of the same width so we pick 'a'

	let glyph_width		= h_advance;
	let glyph_height	= v_advance;

	let cursor_z		= -font.depth_scaled() + (font.depth_scaled() / 4.0);

	let quad_width		= glyph_width;
	let quad_height		= glyph_height;
	let quad_pos		= Vec3::new(0., 0., cursor_z);

	// spawn dedicated quad for cursor
	let quad_entity_id	= 
	quad(
		quad_pos,
		Vec2::new(quad_width, quad_height),
		text_meshes_cache,
		mesh_assets,
		commands
	);

	commands.entity(quad_entity_id).insert(material_handle.clone_weak());

	commands.entity(surface_entity).add_child(quad_entity_id);

	cursor.entity 		= Some(quad_entity_id);
	cursor.color		= cursor_color_fg;
}
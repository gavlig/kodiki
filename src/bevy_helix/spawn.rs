use bevy				:: prelude :: { * };
use bevy_reader_camera	:: { * };
use bevy_contrib_colors	:: { Tailwind };

use super				:: { * };

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
use helix_view :: graphics :: Color as HelixColor;

use crate :: bevy_ab_glyph :: ABGlyphFont;
use crate :: bevy_ab_glyph :: TextMeshesCache;

fn get_quad_mesh_handle(
	quad_mesh_name	: &String,
	quad_size		: Vec2,
	text_mesh_cache	: &mut TextMeshesCache,
	mesh_assets		: &mut Assets<Mesh>,
) -> Handle<Mesh>
{
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
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) -> Entity {
	let quad_mesh_handle = mesh_assets.add(Mesh::from(shape::Quad::new(quad_size)));
	let quad_pos		= quad_pos_in;

	commands.spawn(PbrBundle {
		mesh			: quad_mesh_handle.clone(),
		transform		: Transform {
			translation	: quad_pos,
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

pub fn surface_quad(
	surface_name	: &String,
	surface_bevy	: &mut SurfaceBevy,
	surface_helix	: &SurfaceHelix,
	font			: &ABGlyphFont,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
)
{
	if let Some(background_entity) = surface_bevy.background_entity {
		commands.entity(background_entity).despawn();
	}
	
	let surface_entity = surface_bevy.entity.unwrap();
	surface_bevy.area = surface_helix.area;

	let v_advance	= font.vertical_advance();
	let h_advance	= font.horizontal_advance(&String::from("a")); // in monospace font every letter should be of the same width so we pick 'a'
	let v_down_offset = font.vertical_down_offset();
	
	let width		= surface_helix.area.width;
	let height		= surface_helix.area.height;
	
	let quad_width	= h_advance * width as f32;
	let quad_height	= v_advance * height as f32;
	
	let quad_x		= h_advance * width as f32 / 2.0;
	let mut quad_y	= -v_advance * height as f32 / 2.0;
	// + v_advance because we need to cover row 0 with background quads too
	quad_y 			+= v_advance;
	// add offset downwards to cover glyphs with vertical advance (y, g, _ etc)
	quad_y			-= v_down_offset;
	
	println!("filling surface {} len {} w {} h {}", surface_name, surface_helix.content.len(), width, height);
	
	let quad_pos		= Vec3::new(quad_x, quad_y, -font.depth_scaled());
	let quad_entity_id	= 
	spawn::background_quad(
		quad_pos,
		Vec2::new(quad_width, quad_height),
		mesh_assets,
		commands
	);
	
	surface_bevy.background_entity = Some(quad_entity_id);
	
	commands.entity(surface_entity).add_child(quad_entity_id);
}

pub fn surface(
	surface_name	: &String,
	world_position	: Option<Vec3>,
	surfaces_bevy	: &mut SurfacesMapBevy,
	surface_helix	: &SurfaceHelix,
	font			: &ABGlyphFont,
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
	
	surface_quad(surface_name, &mut surface_bevy, surface_helix, font, mesh_assets, commands);
	
	surfaces_bevy.insert(surface_name.clone(), surface_bevy);
	
	{
		let v_advance	= font.vertical_advance();
		let h_advance	= font.horizontal_advance(&String::from("a")); // in monospace font every letter should be of the same width so we pick 'a'
		
		let width		= surface_helix.area.width;
		let height		= surface_helix.area.height;
		
		let text_descriptor = TextDescriptor {
			rows		: height as u32,
			columns		: width as u32,
			glyph_width	: h_advance,
			glyph_height: v_advance
		};
		
		commands.entity(surface_entity).insert(text_descriptor);
	}

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
	glyph_quad(
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
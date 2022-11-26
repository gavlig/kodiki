use bevy				:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_contrib_colors	:: { Tailwind };

use bevy_prototype_debug_lines :: { * };

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use crate				:: bevy_ab_glyph::{ ABGlyphFont, TextMeshesCache };
use crate				:: bevy_ab_glyph :: mesh_generator :: generate_glyph_mesh_wcache;

use super				:: { * };

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
use helix_tui			:: { buffer :: Cell as CellHelix };

use helix_view			:: { Theme };
use helix_view::graphics::Color as HelixColor;

fn color_from_helix(helix_color: HelixColor) -> Color {
	match helix_color {
		HelixColor::Reset		=> Color::WHITE,
		HelixColor::Black		=> Color::BLACK,
		HelixColor::Red			=> Tailwind::RED600,
		HelixColor::Green		=> Tailwind::GREEN600,
		HelixColor::Yellow		=> Tailwind::YELLOW600,
		HelixColor::Blue		=> Tailwind::BLUE600,
		HelixColor::Magenta		=> Tailwind::PURPLE600,
		HelixColor::Cyan		=> Color::rgb(0.0, 0.5, 0.5),
		HelixColor::Gray		=> Tailwind::GRAY600,
		HelixColor::LightRed	=> Tailwind::RED300,
		HelixColor::LightGreen	=> Tailwind::GREEN300,
		HelixColor::LightBlue	=> Tailwind::BLUE300,
		HelixColor::LightYellow => Tailwind::YELLOW300,
		HelixColor::LightMagenta => Tailwind::PURPLE300,
		HelixColor::LightCyan	=> Color::rgb(0.0, 0.7, 0.7),
		HelixColor::LightGray	=> Tailwind::GRAY300,
		HelixColor::White		=> Color::WHITE,
		// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
		HelixColor::Indexed(_i) => { panic!("Indexed color is not supported!"); }, // Color::AnsiValue(i), 
		HelixColor::Rgb(r, g, b) => Color::rgb_u8(r, g, b),
	}
}

pub fn surface(
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,
	cursor			: &CursorBevy,

	font			: &ABGlyphFont,

	text_mesh_cache	: &mut TextMeshesCache,
	helix_colors_cache : &mut HelixColorsCache,

	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	commands		: &mut Commands,

	mut debug_lines	: &mut DebugLines
)
{
	let root_entity = surface_bevy.entity.unwrap();

	surface_bevy.content.resize_with(surface_helix.content.len(), || { CellBevy::default() });

	let v_advance	= font.vertical_advance();

	let mut surface_children : Vec<Entity> = Vec::new();

	let mut x		= 0.0;
	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 0 as u32;
	
	let width		= surface_helix.area.width;
	let height		= surface_helix.area.height;
	let content_helix = &surface_helix.content;
	let content_bevy = &mut surface_bevy.content;

	for y_cell in 0..height {
		y = -v_advance * row as f32;
		
		for x_cell in 0..width {
			let content_index = (y_cell * width + x_cell) as usize;
			let cell_helix = &content_helix[content_index];
			let cell_bevy = &mut content_bevy[content_index];
			
			// figure out colors first
			let reversed = cell_helix.modifier == helix_view::graphics::Modifier::REVERSED;
			let is_cursor_pos = cursor.x == x_cell && cursor.y == y_cell;
			let wrong_color = !reversed && !is_cursor_pos && (cell_bevy.fg != cell_helix.fg || cell_bevy.bg != cell_helix.bg);
			let reversed_and_wrong_color = reversed && !is_cursor_pos && (cell_bevy.fg != cell_helix.bg || cell_bevy.bg != cell_helix.fg);
			let in_cursor_pos_and_wrong_color = is_cursor_pos && cell_bevy.fg != cell_helix.bg;

			if wrong_color || reversed_and_wrong_color || in_cursor_pos_and_wrong_color {
				update_cell_materials(
					cell_bevy,
					cell_helix,
					reversed,
					is_cursor_pos,
					helix_colors_cache,
					material_assets,
					commands
				);
			}

			// now spawn new mesh if needed
			let pos = Vec3::new(x, y, 0.0);

			let wrong_symbol = cell_helix.symbol != cell_bevy.symbol;
			if wrong_symbol {
				if y_cell == 0 {
					println!("[{} {}] [{} {}] wrong symbol [{}] <= [{}]", x_cell, y_cell, x, y, cell_helix.symbol, cell_bevy.symbol);
				}

				update_cell_mesh(
					pos,
					y_cell,
					cell_helix,
					cell_bevy,
					font,
					text_mesh_cache,
					&mut surface_children,
					mesh_assets,
					commands
				);

				// print!("\n");
			}

			x += font.horizontal_advance(&cell_helix.symbol);

			column += 1;
		}

		x			= 0.0;

		column		= 0;
		row			+= 1;
	}

	if surface_children.len() > 0 {
		commands.entity(root_entity).push_children(surface_children.as_slice());
	}
}

fn update_cell_mesh(
	pos				: Vec3,
	y_cell			: u16,
	cell_helix		: &CellHelix,
	cell_bevy		: &mut CellBevy,
	font			: &ABGlyphFont,
	text_meshes_cache : &mut TextMeshesCache,
	surface_children: &mut Vec<Entity>,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) {
	// Special case - space character. Doesn't require mesh
	let space_symbol = cell_helix.symbol == " ";
	if space_symbol {
		// remove a mesh if there was an entity
		if let Some(entity) = cell_bevy.symbol_entity {
			// remove mesh
			commands.entity(entity).remove::<Handle<Mesh>>();
			if y_cell == 0 {
				println!("removing mesh because new symbol is space! was {}", cell_bevy.symbol);
			}
		}
		cell_bevy.symbol = cell_helix.symbol.clone();
		return;
	}

	let mesh_handle = generate_glyph_mesh_wcache(
		&cell_helix.symbol,
		&font,
		mesh_assets,
		text_meshes_cache
	);

	if let Some(entity) = cell_bevy.symbol_entity {
		if y_cell == 0 {
			println!("replacing mesh handle for [{}]", cell_helix.symbol);
		}

		// replace previous mesh with new one
		commands.entity(entity)
			.remove::<Handle<Mesh>>()
			.insert(mesh_handle)
			;
	} else {
		if y_cell == 0 {
			println!("spawning new entity with an existing mesh for [{}] pos: {:?}", cell_helix.symbol, pos);
		}

		// spawn new entity with an existing mesh
		cell_bevy.symbol_entity = Some(
			commands.spawn_bundle(PbrBundle {
				mesh : mesh_handle,
				material : cell_bevy.fg_handle.as_ref().unwrap().clone_weak(),
				transform : Transform {
					translation	: pos,
					scale		: [font.scale; 3].into(),
					..default()
				},
				..default()
			})
			.id()
		);

		// insert material
		commands.entity(cell_bevy.symbol_entity.unwrap()).insert(
			cell_bevy.fg_handle.as_ref().unwrap().clone_weak()
		);

		surface_children.push(cell_bevy.symbol_entity.unwrap());
	}

	cell_bevy.symbol = cell_helix.symbol.clone();
}

pub fn update_cell_materials(
	cell_bevy: &mut CellBevy,
	cell_helix: &CellHelix,
	reversed: bool,
	is_cursor_pos: bool,
	helix_colors_cache: &mut HelixColorsCache,
	material_assets: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	// first take care of reversed colors: if reversed foreground becomes background
	(cell_bevy.fg, cell_bevy.bg) =
	if !reversed {
		(cell_helix.fg, cell_helix.bg)
	} else {
		(cell_helix.bg, cell_helix.fg)
	};
	
	// emulate reversed color only for symbol because native cursor rendering is turned off
	if is_cursor_pos {
		cell_bevy.fg = cell_helix.bg;
	}

	let color_fg = color_from_helix(cell_bevy.fg);
	let color_bg = color_from_helix(cell_bevy.bg);

	cell_bevy.fg_handle = Some(get_helix_color_material_handle(
		color_fg,
		&mut helix_colors_cache.materials,
		material_assets
	));

	cell_bevy.bg_handle = Some(get_helix_color_material_handle(
		color_bg,
		&mut helix_colors_cache.materials,
		material_assets
	));

	// replace material to reflect changed color
	if let Some(cell_bevy_entity_symbol) = cell_bevy.symbol_entity {
		commands.entity		(cell_bevy_entity_symbol)
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.fg_handle.as_ref().unwrap().clone_weak())
		;
	}
	
	if let Some(cell_bevy_entity_bg_quad) = cell_bevy.entity_bg_quad {
		commands.entity		(cell_bevy_entity_bg_quad)
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.bg_handle.as_ref().unwrap().clone_weak())
		;
	}
}

pub fn cursor(
	surface_bevy	: &SurfaceBevy,
	font			: &ABGlyphFont,
	cursor			: &mut CursorBevy,
	q_cursor_transform : &mut Query<&mut Transform>,
	time			: &Res<Time>,
	theme			: &Theme,
	text_meshes_cache : &mut TextMeshesCache,
	helix_colors_cache : &mut MaterialsMap,
	material_assets	: &mut Assets<StandardMaterial>,
	mesh_assets		: &mut ResMut<Assets<Mesh>>,
	commands		: &mut Commands
)
{
	let cursor_theme = theme.get("ui.cursor");
	if cursor_theme.bg.is_none() {
		return;
	}

	let root_entity 		= surface_bevy.entity.unwrap();

	let v_advance			= font.vertical_advance();
	let h_advance			= font.horizontal_advance(&String::from("a")); // in monospace font every letter should be of the same width so we pick 'a'
	let v_down_offset		= font.vertical_down_offset();

	let glyph_width			= h_advance;
	let glyph_height		= v_advance;

	let cursor_z			= -font.depth_scaled() + (font.depth_scaled() / 4.0);

	// move background quad
	if cursor.entity.is_some() && cursor.easing_accum < 1.0 {
		let column_offset 	= (cursor.x as f32) * h_advance;
		let row_offset		= (cursor.y as f32) * -v_advance + v_advance; 

		let target_x 		= column_offset	+ (glyph_width / 2.0);
		let target_y 		= row_offset	- (glyph_height / 2.0) - v_down_offset;

		let target_pos		= Vec3::new(target_x, target_y, cursor_z);

		let delta_seconds	= time.delta_seconds();
		let delta_accum		= delta_seconds / /*cursor_easing_seconds*/0.05;

		let cursor_entity 	= cursor.entity.unwrap();
		let mut cursor_transform = q_cursor_transform.get_mut(cursor_entity).unwrap();

		cursor.easing_accum = (cursor.easing_accum + delta_accum).min(1.0);
		cursor_transform.translation = cursor_transform.translation.lerp(target_pos, cursor.easing_accum);
	}
	
	let cursor_color_fg		= color_from_helix(theme.get("ui.cursor").bg.unwrap());
	let material_handle		= get_helix_color_material_handle(cursor_color_fg, helix_colors_cache, material_assets);
	
	// spawn background quad for cursor
	if cursor.entity == None {
		let quad_width		= glyph_width;
		let quad_height		= glyph_height;
		let quad_pos		= Vec3::new(0., 0., cursor_z);

		let quad_entity_id	= 
		super::spawn::quad(
			quad_pos,
			Vec2::new(quad_width, quad_height),
			text_meshes_cache,
			mesh_assets,
			commands
		);

		commands.entity(quad_entity_id).insert(material_handle.clone_weak());

		commands.entity(root_entity).add_child(quad_entity_id);

		cursor.entity 		= Some(quad_entity_id);
		cursor.color		= cursor_color_fg;
	} else if cursor.color != cursor_color_fg {
		commands.entity		(cursor.entity.unwrap())
		.remove::<Handle<StandardMaterial>>()
		.insert(material_handle.clone_weak())
		;
		cursor.color		= cursor_color_fg;
	}
}
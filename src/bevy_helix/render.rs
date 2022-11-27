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

	font			: &ABGlyphFont,

	text_meshes_cache : &mut TextMeshesCache,
	helix_colors_cache : &mut HelixColorsCache,

	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	commands		: &mut Commands,
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
			update_cell_materials(
				cell_bevy,
				cell_helix,
				helix_colors_cache,
				material_assets,
				commands
			);

			// now spawn new mesh if needed
			let pos = Vec3::new(x, y, 0.0);
			update_cell_mesh(
				pos,
				cell_helix,
				cell_bevy,
				font,
				text_meshes_cache,
				&mut surface_children,
				mesh_assets,
				commands
			);

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
	cell_helix		: &CellHelix,
	cell_bevy		: &mut CellBevy,
	font			: &ABGlyphFont,
	text_meshes_cache : &mut TextMeshesCache,
	surface_children: &mut Vec<Entity>,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
) {
	let wrong_symbol = cell_helix.symbol != cell_bevy.symbol;
	if !wrong_symbol {
		return;
	}

	// Special case - space character. Doesn't require mesh
	let space_symbol = cell_helix.symbol == " ";
	if space_symbol {
		// remove a mesh if there was an entity
		if let Some(entity) = cell_bevy.symbol_entity {
			// remove mesh
			commands.entity(entity).remove::<Handle<Mesh>>();
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
		// replace previous mesh with new one
		commands.entity(entity)
			.remove::<Handle<Mesh>>()
			.insert(mesh_handle)
			;
	} else {
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
	helix_colors_cache: &mut HelixColorsCache,
	material_assets: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	let reversed = cell_helix.modifier == helix_view::graphics::Modifier::REVERSED;
	let wrong_color = !reversed && (cell_bevy.fg != cell_helix.fg || cell_bevy.bg != cell_helix.bg);
	let reversed_and_wrong_color = reversed && (cell_bevy.fg != cell_helix.bg || cell_bevy.bg != cell_helix.fg);

	if !wrong_color && !reversed_and_wrong_color {
		return;
	}

	// take care of reversed colors: if reversed - foreground becomes background
	(cell_bevy.fg, cell_bevy.bg) =
	if !reversed {
		(cell_helix.fg, cell_helix.bg)
	} else {
		(cell_helix.bg, cell_helix.fg)
	};
	
	update_cell_materials_inner(
		cell_bevy,
		helix_colors_cache,
		material_assets,
		commands
	);
}

fn update_cell_materials_inner(
	cell_bevy: &mut CellBevy,
	helix_colors_cache: &mut HelixColorsCache,
	material_assets: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	let color_fg = color_from_helix(cell_bevy.fg);
	let color_bg = color_from_helix(cell_bevy.bg);

	cell_bevy.fg_handle = Some(get_helix_color_material_handle(
		color_fg,
		helix_colors_cache,
		material_assets
	));

	cell_bevy.bg_handle = Some(get_helix_color_material_handle(
		color_bg,
		helix_colors_cache,
		material_assets
	));

	// replace material to reflect changed color
	if let Some(cell_bevy_entity_symbol) = cell_bevy.symbol_entity {
		commands.entity		(cell_bevy_entity_symbol)
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.fg_handle.as_ref().unwrap().clone_weak())
		;
	}
	
	if let Some(cell_bevy_entity_bg_quad) = cell_bevy.bg_quad_entity {
		commands.entity		(cell_bevy_entity_bg_quad)
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.bg_handle.as_ref().unwrap().clone_weak())
		;
	}
}

pub fn cursor(
	cursor			: &mut CursorBevy,
	
	surface_bevy	: &mut SurfaceBevy,
	surface_helix	: &SurfaceHelix,
	theme			: &Theme,

	helix_colors_cache : &mut HelixColorsCache,

	material_assets	: &mut Assets<StandardMaterial>,
	commands		: &mut Commands
)
{
	let cursor_theme = theme.get("ui.cursor");
	if cursor_theme.bg.is_none() {
		return;
	}

	let cursor_color_fg		= color_from_helix(cursor_theme.bg.unwrap());
	let material_handle		= get_helix_color_material_handle(cursor_color_fg, helix_colors_cache, material_assets);
	
	if cursor.color != cursor_color_fg {
		commands.entity		(cursor.entity.unwrap())
		.remove::<Handle<StandardMaterial>>()
		.insert(material_handle.clone_weak())
		;
		cursor.color		= cursor_color_fg;
	}

	let width				= surface_helix.area.width;
	let content_helix		= &surface_helix.content;
	let content_bevy		= &mut surface_bevy.content;

	let content_index 		= (cursor.y * width + cursor.x) as usize;
	let cell_helix			= &content_helix[content_index];
	let cell_bevy			= &mut content_bevy[content_index];

	update_cursor_cell_material(
		cell_bevy,
		cell_helix,
		helix_colors_cache,
		material_assets,
		commands
	)
}

pub fn update_cursor_cell_material(
	cell_bevy: &mut CellBevy,
	cell_helix: &CellHelix,
	helix_colors_cache: &mut HelixColorsCache,
	material_assets: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	let wrong_color = (cell_bevy.fg != cell_helix.fg && cell_bevy.bg != cell_helix.bg);
	if !wrong_color {
		return;
	}

	// helix reverses color in cell with cursor and we "revert" it back to make it visible with 3d cursor
	cell_bevy.fg = cell_helix.fg;
	cell_bevy.bg = cell_helix.bg;

	update_cell_materials_inner(
		cell_bevy,
		helix_colors_cache,
		material_assets,
		commands
	);
}
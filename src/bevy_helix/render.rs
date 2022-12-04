use bevy				:: prelude :: { * };
use bevy_contrib_colors	:: { Tailwind };

use crate				:: bevy_ab_glyph::{ UsedFonts, GlyphWithFonts, TextMeshesCache };
use crate				:: bevy_ab_glyph :: mesh_generator :: { generate_string_mesh_wcache };

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

struct Word<'a> {
	pub x				: f32,
	pub y				: f32,
	pub row				: u32,
	pub column			: u32,
	pub color			: HelixColor,
	pub string_with_fonts : StringWithFonts<'a>,
	pub string			: String,
}

impl Default for Word<'_> {
	fn default() -> Self {
		Self {
			x			: 0.0,
			y			: 0.0,
			row			: 0,
			column		: 0,
			color		: HelixColor::Cyan,
			string_with_fonts : Vec::new(),
			string		: String::new(),
		}
	}
}

impl Word<'_> {

}

type Words<'a> = Vec<Word<'a>>;

#[derive(Component, Clone, Debug)]
pub struct WordDescription {
	pub string	: String,
	pub row		: u32,
	pub column	: u32,
}

pub fn surface(
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,

	used_fonts		: &UsedFonts,

	text_meshes_cache : &mut TextMeshesCache,
	helix_colors_cache : &mut HelixColorsCache,

	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	commands		: &mut Commands,
)
{
	if !surface_bevy.update {
		return;
	}
	
	surface_bevy.rows.resize_with(surface_helix.area.height as usize, || { WordRowBevy::new() });
	
	let root_entity = surface_bevy.entity.unwrap();

	let v_advance	= used_fonts.main.vertical_advance();

	let mut surface_children : Vec<Entity> = Vec::new();

	let mut x		= 0.0;
	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 0 as u32;
	
	let width		= surface_helix.area.width;
	let height		= surface_helix.area.height;
	let cells_helix = &surface_helix.content;
	let rows_bevy	= &mut surface_bevy.rows;

	for y_cell in 0..height {
		y = -v_advance * row as f32;
		
		let mut row_bevy		= &mut rows_bevy[y_cell as usize];
		let mut row_synced		= true;
		let mut words			= Words::new();
		let mut word_started	= false;
		
		for x_cell in 0..width {
			let content_index	= (y_cell * width + x_cell) as usize;
			let cell_helix		= &cells_helix[content_index];
			let glyph_with_fonts_current = GlyphWithFonts::new(cell_helix.symbol.clone(), used_fonts);
			
			// collect symbols with the same font/color into a word and spawn it when word ends
			let symbol_color	= cell_helix.fg;
			let is_space		= cell_helix.symbol == " " || cell_helix.symbol == "\t";
			let end_of_row		= x_cell == width - 1;
			
			// check if word ended and if not then add symbol to the word in progress
			if word_started {
				let word_index	= words.len() - 1;
				let word		= words.last_mut().unwrap();
				let glyph_with_fonts_current = GlyphWithFonts::new(cell_helix.symbol.clone(), used_fonts);
				
				let different_color = word.color != symbol_color;
				let different_font	= if let Some(char_with_fonts) = word.string_with_fonts.first() {
					char_with_fonts.current_font() != glyph_with_fonts_current.current_font()
				} else {
					false
				};
				
				// if word ended check if it's different from what we already have spawned and spawn it or re-use existing entity to attach a different mesh to it
				let word_ended	= is_space || different_color || different_font || end_of_row;
				if word_ended {
					word_started = false;
					
					if row_synced || word_index == 0 {
						row_synced = check_row_sync(word_index, word, &mut row_bevy, commands);
					}
					
					let word_description = WordDescription {
						string	: word.string.clone(),
						row		: row,
						column	: column,
					};
					
					// now spawn new mesh if needed
					if !row_synced {
						let word_entity = update_word_mesh(
							word_index,
							word,
							&word_description,
							&mut row_bevy,
							text_meshes_cache,
							helix_colors_cache,
							mesh_assets,
							material_assets,
							commands
						);
						
						if let Some(entity) = word_entity {
							surface_children.push(entity);
						}
					}
				} else {
					word.string.push_str(cell_helix.symbol.as_str());
					word.string_with_fonts.push(glyph_with_fonts_current);
				}
			}
			
			if !is_space && !word_started {
				word_started	= true;
				
				let mut word	= Word::default();
				word.x			= x;
				word.y			= y;
				word.row		= row;
				word.column		= column;
				word.color		= symbol_color;
				
				word.string.push_str(cell_helix.symbol.as_str());
				word.string_with_fonts.push(glyph_with_fonts_current);
				
				words.push		(word);
			}
			
			if end_of_row && (!row_synced || words.len() == 0) {
				let word_index	= words.len();
				cleanup_desync_row(word_index, &mut row_bevy, commands);
			}
			
			x += used_fonts.main.horizontal_advance(&cell_helix.symbol);

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

fn check_row_sync(
	word_index			: usize,
	word 				: &Word,
	row_bevy			: &mut WordRowBevy,
	commands			: &mut Commands
) -> bool
{
	let row_len			= row_bevy.len();
	if word_index >= row_len {
		return false;
	}

	// check if it's the same word as we already have in row_bevy and return if so
	let word_bevy = &row_bevy[word_index];
	if word_bevy.string == word.string && word_bevy.column == word.column && word_bevy.color == word.color {
		return true;
	}
	
	// as we're desynced invalidate all remaining meshes, transforms and materials. Just keep entities to avoid respawning
	// TODO: we can be smarter here and clean up only current word since next word can be valid just with wrong transform and/or material
	for i in word_index .. row_len {
		let word_bevy = &row_bevy[i];
		commands.entity(word_bevy.entity.unwrap())
		.remove::<Handle<Mesh>>()
		.remove::<Handle<StandardMaterial>>()
		.remove::<Transform>()
		.remove::<WordDescription>()
		;
	}
	
	return false;
}

fn cleanup_desync_row(
	word_index			: usize,
	row_bevy			: &mut WordRowBevy,
	commands			: &mut Commands
)
{
	let row_len			= row_bevy.len();
	if word_index >= row_len {
		return;
	}
	
	for i in word_index .. row_len {
		let word_bevy = &row_bevy[i];
		commands.entity(word_bevy.entity.unwrap()).despawn_recursive();
	}
	
	assert!(word_index <= row_len);
	row_bevy.truncate(word_index);
}

fn update_word_mesh(
	word_index			: usize,
	word 				: &Word,
	word_description	: &WordDescription,
	row_bevy			: &mut WordRowBevy,
	text_meshes_cache	: &mut TextMeshesCache,
	helix_colors_cache	: &mut HelixColorsCache,
	mesh_assets			: &mut Assets<Mesh>,
	material_assets		: &mut Assets<StandardMaterial>,
	commands			: &mut Commands
) -> Option<Entity>
{
	let word_mesh_handle = generate_string_mesh_wcache(&word.string_with_fonts, mesh_assets, text_meshes_cache);
	let color			= color_from_helix(word.color);
	let material_handle = get_helix_color_material_handle(
		color,
		helix_colors_cache,
		material_assets
	);
	
	// spawn new word if it doesnt exist in the row yet
	if word_index >= row_bevy.len() {
		let word_entity = spawn_word_mesh(
			word,
			word_description,
			&word_mesh_handle,
			&material_handle,
			commands
		);
		
		row_bevy.push(WordBevy {
			entity		: Some(word_entity),
			string		: word.string.clone(),
			color		: word.color,
			column		: word.column
		});
		
		return Some(word_entity);
	} else {
		let word_bevy = &mut row_bevy[word_index];
		
		word_bevy.string = word.string.clone();
		word_bevy.color	= word.color;
		word_bevy.column = word.column;
		
		let entity = word_bevy.entity.unwrap();
		fill_word_entity(entity, word, word_description, &word_mesh_handle, &material_handle, commands);
		
		return None;
	}
}

fn spawn_word_mesh(
	word 			: &Word,
	word_description: &WordDescription,
	mesh_handle		: &Handle<Mesh>,
	material_handle	: &Handle<StandardMaterial>,
	commands		: &mut Commands
) -> Entity
{
	let font		= word.string_with_fonts.first().unwrap().current_font();
	let word_mesh_entity = commands.spawn(PbrBundle {
		mesh		: mesh_handle.clone_weak(),
		material	: material_handle.clone_weak(),
		transform	: Transform {
			translation	: Vec3::new(word.x, word.y, 0.0),
			scale		: [font.scale; 3].into(),
			..default()
		},
		..default()
	})
	.insert(word_description.clone())
	.id();
	
	word_mesh_entity
}

fn fill_word_entity(
	entity			: Entity,
	word 			: &Word,
	word_description: &WordDescription,
	mesh_handle		: &Handle<Mesh>,
	material_handle	: &Handle<StandardMaterial>,
	commands		: &mut Commands
)
{
	let font		= word.string_with_fonts.first().unwrap().current_font();
	let transform	= Transform {
		translation	: Vec3::new(word.x, word.y, 0.0),
		scale		: [font.scale; 3].into(),
		..default()
	};
	
	commands.entity(entity)
	.insert(mesh_handle.clone_weak())
	.insert(material_handle.clone_weak())
	.insert(transform)
	.insert(word_description.clone())
	;
}

// fn update_cell_background(
// 	cell_bevy: &mut CellBevy,
// 	cell_helix: &CellHelix,
// 	helix_colors_cache: &mut HelixColorsCache,
// 	material_assets: &mut Assets<StandardMaterial>,
// 	commands: &mut Commands
// ) {
// 	let reversed = cell_helix.modifier == helix_view::graphics::Modifier::REVERSED;
// 	let wrong_color = !reversed && (cell_bevy.fg != cell_helix.fg || cell_bevy.bg != cell_helix.bg);
// 	let reversed_and_wrong_color = reversed && (cell_bevy.fg != cell_helix.bg || cell_bevy.bg != cell_helix.fg);

// 	if !wrong_color && !reversed_and_wrong_color {
// 		return;
// 	}

// 	// take care of reversed colors: if reversed - foreground becomes background
// 	(cell_bevy.fg, cell_bevy.bg) =
// 	if !reversed {
// 		(cell_helix.fg, cell_helix.bg)
// 	} else {
// 		(cell_helix.bg, cell_helix.fg)
// 	};
	
// 	update_cell_background_inner(
// 		cell_bevy,
// 		helix_colors_cache,
// 		material_assets,
// 		commands
// 	);
// }

// fn update_cell_background_inner(
// 	cell_bevy: &mut CellBevy,
// 	helix_colors_cache: &mut HelixColorsCache,
// 	material_assets: &mut Assets<StandardMaterial>,
// 	commands: &mut Commands
// ) {
// 	let color_fg = color_from_helix(cell_bevy.fg);
// 	let color_bg = color_from_helix(cell_bevy.bg);

// 	cell_bevy.fg_handle = Some(get_helix_color_material_handle(
// 		color_fg,
// 		helix_colors_cache,
// 		material_assets
// 	));

// 	cell_bevy.bg_handle = Some(get_helix_color_material_handle(
// 		color_bg,
// 		helix_colors_cache,
// 		material_assets
// 	));

// 	// replace material to reflect changed color
// 	if let Some(cell_bevy_entity_symbol) = cell_bevy.symbol_entity {
// 		commands.entity		(cell_bevy_entity_symbol)
// 		.remove::<Handle<StandardMaterial>>()
// 		.insert(cell_bevy.fg_handle.as_ref().unwrap().clone_weak())
// 		;
// 	}
	
// 	if let Some(cell_bevy_entity_bg_quad) = cell_bevy.bg_quad_entity {
// 		commands.entity		(cell_bevy_entity_bg_quad)
// 		.remove::<Handle<StandardMaterial>>()
// 		.insert(cell_bevy.bg_handle.as_ref().unwrap().clone_weak())
// 		;
// 	}
// }

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

	// let width				= surface_helix.area.width;
	// let content_helix		= &surface_helix.content;
	// let content_bevy		= &mut surface_bevy.content;

	// let content_index 		= (cursor.y * width + cursor.x) as usize;
	// let cell_helix			= &content_helix[content_index];
	// let cell_bevy			= &mut content_bevy[content_index];

	// update_cursor_cell_material(
	// 	cell_bevy,
	// 	cell_helix,
	// 	helix_colors_cache,
	// 	material_assets,
	// 	commands
	// )
}

// fn update_cursor_cell_material(
// 	cell_bevy: &mut CellBevy,
// 	cell_helix: &CellHelix,
// 	helix_colors_cache: &mut HelixColorsCache,
// 	material_assets: &mut Assets<StandardMaterial>,
// 	commands: &mut Commands
// ) {
// 	let wrong_color = (cell_bevy.fg != cell_helix.fg && cell_bevy.bg != cell_helix.bg);
// 	if !wrong_color {
// 		return;
// 	}

// 	// helix reverses color in cell with cursor and we "revert" it back to make it visible with 3d cursor
// 	cell_bevy.fg = cell_helix.fg;
// 	cell_bevy.bg = cell_helix.bg;

// 	update_cell_background_inner(
// 		cell_bevy,
// 		helix_colors_cache,
// 		material_assets,
// 		commands
// 	);
// }
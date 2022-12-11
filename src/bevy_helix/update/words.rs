use bevy				:: prelude :: { * };

use crate				:: bevy_ab_glyph::{ UsedFonts, GlyphWithFonts, TextMeshesCache };
use crate				:: bevy_ab_glyph :: mesh_generator :: { generate_string_mesh_wcache };

use super				:: { * };

use helix_tui			:: { buffer :: Cell as CellHelix };

use helix_view::graphics::Color as HelixColor;

pub struct Word<'a> {
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

pub type Row<'a> = Vec<Word<'a>>;

#[derive(Default)]
pub struct RowState {
	pub word_started : bool,
	pub synced		: bool,
	pub ended		: bool,
}

#[derive(Component, Clone, Debug)]
pub struct WordDescription {
	pub string	: String,
	pub row		: u32,
	pub column	: u32,
}

pub fn update<'a>(
	table_coords	: &TableCoords,
	row_bevy		: &mut WordRowBevy,
	row_state		: &mut RowState,
	
	words_row		: &mut Row<'a>,
	cell_helix		: &CellHelix,
	used_fonts		: &'a UsedFonts<'a>,
	
	text_meshes_cache: &mut TextMeshesCache,
	helix_colors_cache: &mut HelixColorsCache,
	
	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	
	commands		: &mut Commands,
) -> Vec<Entity>
{
	let symbol_color	= cell_helix.fg;
	let is_space		= cell_helix.symbol == " " || cell_helix.symbol == "\t";
	
	let mut word_entities : Vec<Entity> = Vec::new();
	let glyph_with_fonts_current = GlyphWithFonts::new(cell_helix.symbol.clone(), used_fonts);
	
    if row_state.word_started {
		let word_index	= words_row.len() - 1;
		let word		= words_row.last_mut().unwrap();
	
		let different_color = word.color != symbol_color;
		let different_font	= if let Some(char_with_fonts) = word.string_with_fonts.first() {
			char_with_fonts.current_font() != glyph_with_fonts_current.current_font()
		} else {
			false
		};
	
		// if word ended check if it's different from what we already have spawned and spawn it or re-use existing entity to attach a different mesh to it
		let word_ended	= is_space || different_color || different_font || row_state.ended;
		
		if (word_ended && row_state.ended && !is_space) || !word_ended {
			word.string.push_str(cell_helix.symbol.as_str());
			word.string_with_fonts.push(glyph_with_fonts_current.clone());
		}
		
		if word_ended {
			let entity = on_word_ended(
				word_index,
				word,
				table_coords,
				row_bevy,
				row_state,
				text_meshes_cache,
				helix_colors_cache,
				mesh_assets,
				material_assets,
				commands
			);
			
			if let Some(to_add) = entity {
				word_entities.push(to_add)
			}
		}
	}
	
    if !is_space && !row_state.word_started {
		row_state.word_started = true;
	
		let mut word	= Word::default();
		word.x			= table_coords.x;
		word.y			= table_coords.y;
		word.row		= table_coords.row;
		word.column		= table_coords.column;
		word.color		= symbol_color;
	
		word.string.push_str(cell_helix.symbol.as_str());
		word.string_with_fonts.push(glyph_with_fonts_current.clone());
	
		if row_state.ended {
			let entity = on_word_ended(
				words_row.len(),
				&word,
				table_coords,
				row_bevy,
				row_state,
				text_meshes_cache,
				helix_colors_cache,
				mesh_assets,
				material_assets,
				commands
			);
			
			if let Some(to_add) = entity {
				word_entities.push(to_add)
			}
		}

		words_row.push		(word);
	}
	
	let words_cnt		= words_row.len();
    if row_state.ended && (!row_state.synced || words_cnt == 0 || words_cnt < row_bevy.len()) {
		cleanup_word_row_from(words_cnt, row_bevy, commands);
	}
	
	return word_entities;
}

fn on_word_ended(
	word_index		: usize,	
	word 			: &Word,
	table_coords	: &TableCoords,
	row_bevy		: &mut WordRowBevy,
	row_state		: &mut RowState,
	
	text_meshes_cache: &mut TextMeshesCache,
	helix_colors_cache: &mut HelixColorsCache,
	
	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	
	commands		: &mut Commands,
) -> Option<Entity>
{
	let mut word_entity : Option<Entity> = None;
	
	row_state.word_started = false;
		
	if row_state.synced || word_index == 0 {
		row_state.synced = check_word_row_sync(word_index, word, row_bevy, commands);
	}

	let word_description = WordDescription {
		string	: word.string.clone(),
		row		: table_coords.row,
		column	: table_coords.column,
	};

	// now spawn new mesh if needed
	if !row_state.synced {
		word_entity = update_word(
			word_index,
			word,
			&word_description,
			row_bevy,
			text_meshes_cache,
			helix_colors_cache,
			mesh_assets,
			material_assets,
			commands
		);
	}
	
	word_entity
}

fn check_word_row_sync(
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

fn cleanup_word_row_from(
	word_index_from		: usize,
	row_bevy			: &mut WordRowBevy,
	commands			: &mut Commands
)
{
	let row_len			= row_bevy.len();
	if word_index_from >= row_len {
		return;
	}
	
	for i in word_index_from .. row_len {
		let word_bevy	= &row_bevy[i];
		commands.entity	(word_bevy.entity.unwrap()).despawn_recursive();
	}
	
	assert!(word_index_from <= row_len);
	row_bevy.truncate(word_index_from);
}

fn update_word(
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
	
	// spawn new word if row doesnt have entity yet
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
	// replace word mesh otherwise
	} else {
		let word_bevy = &mut row_bevy[word_index];
		
		word_bevy.string = word.string.clone();
		word_bevy.color	= word.color;
		word_bevy.column = word.column;
		
		let entity = word_bevy.entity.unwrap();
		insert_word_mesh(entity, word, word_description, &word_mesh_handle, &material_handle, commands);
		
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

fn insert_word_mesh(
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
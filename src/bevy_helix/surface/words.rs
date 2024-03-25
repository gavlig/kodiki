use bevy :: prelude :: *;

use helix_tui :: buffer :: Cell as CellHelix;

use super :: *;

use crate :: bevy_ab_glyph :: GlyphWithFonts;

#[derive(Default)]
pub struct RowState {
	pub word_started	: bool,
	pub ended			: bool,
}

pub fn append_symbol<'a>(
	surface_name		: &String,
	surface_is_editor	: bool,
	surface_coords		: &SurfaceCoords,
	cell_helix			: &CellHelix,

	new_row				: &mut WordRow,
	new_row_state		: &mut RowState,
	fonts				: &'a ABGlyphFonts<'a>,
) {
	let symbol_color	= color_from_helix(cell_helix.fg);
	let is_space		= cell_helix.symbol == " " || cell_helix.symbol == "\t";

	let punctuation_check = |c: char| -> bool { c.is_ascii_punctuation() && c != '_' };
	let numeric_check	= |c: char| -> bool { c.is_numeric() };
	let is_punctuation	= cell_helix.symbol.chars().all(&punctuation_check);
	let is_numeric		= cell_helix.symbol.chars().all(&numeric_check);

	let new_glyph_with_fonts = GlyphWithFonts::new(&cell_helix.symbol, fonts);

	// add new symbol to the last word in new row if it's not space
	if new_row_state.word_started {
		let word		= new_row.last_mut().unwrap();

		let different_color = word.color != symbol_color;

		let first_glyph_in_word		= String::from(word.string.chars().next().unwrap());
		let first_glyph_with_fonts	= GlyphWithFonts::new(&first_glyph_in_word, fonts);

		let different_font			= first_glyph_with_fonts.current_font() != new_glyph_with_fonts.current_font();
		let is_emoji				= new_glyph_with_fonts.is_emoji;
		let separate_punctuation	= (is_punctuation && !word.is_punctuation) || (!is_punctuation && word.is_punctuation);
		let separate_numeric		= !is_numeric && word.is_numeric;

		// if word ended check if it's different from what we already have cached and replace it with new one if so
		let word_ended = is_space || different_color || different_font || is_emoji || separate_punctuation || separate_numeric;

		// add new symbol to the word that we started filling
		if !word_ended {
			word.string.push_str(cell_helix.symbol.as_str());
		} else {
			new_row_state.word_started = false;
		}
	}

	// if new symbol is not space and we haven't started collecting another word start collecting now
	if !is_space && !new_row_state.word_started {
		new_row_state.word_started = true;

		let new_word = WordDescription {
			x					: surface_coords.x,
			y					: surface_coords.y,
			row					: surface_coords.row_index_global(),
			column				: surface_coords.column,
			word_index			: new_row.len(),
			cached_row_index	: surface_coords.row_index_wcache() as usize,
			surface_name		: surface_name.clone(),
			color				: symbol_color,
			string				: cell_helix.symbol.clone(),
			is_on_editor		: surface_is_editor,
			is_punctuation,
			is_numeric,
			..default()
		};

		new_row.push(new_word);
	}

	// every emoji symbol is a separate word
	if new_glyph_with_fonts.is_emoji {
		new_row_state.word_started = false;
	}
}

#[cfg(feature = "word_spawn_debug")]
fn word_row_debug_log(row: &WordRow) -> (String, String) {
	let mut row_strings = String::new();
	let mut row_entities = String::new();
	for (index, word) in row.iter().enumerate() {
		row_strings.push_str(format!("[{}]\"{}\" ", index, word.string).as_str());
		row_entities.push_str(format!("[{}]{:?} ", index, word.entity).as_str());
	}

	(row_entities, row_strings)
}

pub fn update_cached_row(
	cached_row			: &mut WordRow,
	new_row				: &WordRow,
	to_spawn			: &mut WordsToSpawn,
	to_despawn			: &mut DespawnResource,
	_surface_coords		: &SurfaceCoords
) {
	let mut _row_changed = false;

	#[cfg(feature = "word_spawn_debug")]
	let mut debug_log	= String::new();

	#[cfg(feature = "word_spawn_debug")] {
		debug_log.push_str(format!("update_cached_row [{}] row [{}] len: {}\n", _surface_coords.row_index_wcache(), _surface_coords.row, new_row.len()).as_str());
		let (row_strings, row_entities) = word_row_debug_log(cached_row);
		debug_log.push_str(format!("cached_row before:\nstring: {}\nentities: {}\n", row_strings, row_entities).as_str());
	}

	let cached_row_len = cached_row.len();

	for (new_word_index, new_word) in new_row.iter().enumerate() {
		if new_word_index >= cached_row_len {
			cached_row.push(new_word.clone());
			add_word_to_spawn(new_word, to_spawn);
			continue;
		}

		let cached_word = &cached_row[new_word_index];
		if word_has_changed(new_word, cached_word) || cached_word.entity.is_none() {
			update_cached_word(new_word_index, new_word, cached_row, to_spawn, to_despawn);

			_row_changed = true;
		}
	}

	let new_row_len = new_row.len();
    if new_row_len == 0 || new_row_len < cached_row_len {
		cleanup_cached_row_from(new_row_len, cached_row, to_despawn);
	}

	_row_changed |= new_row_len != cached_row_len;

	#[cfg(feature = "word_spawn_debug")] {
		let (row_strings, row_entities) = word_row_debug_log(cached_row);
		debug_log.push_str(format!("cached_row after:\nstring: {}\nentities: {}\n", row_strings, row_entities).as_str());
		if _row_changed {
			println!("{}", debug_log);
		}
	}
}

fn add_word_to_spawn(new_word: &WordDescription, words_to_spawn: &mut WordsToSpawn) {
	if let Some((_, descriptions)) = words_to_spawn.per_surface.get_key_value_mut(&new_word.surface_name) {
		descriptions.push(new_word.clone());
	} else {
		words_to_spawn.per_surface.insert_unique_unchecked(new_word.surface_name.clone(), [new_word.clone()].into());
	}
}

fn word_has_changed(
	new_word			: &WordDescription,
	cached_word			: &WordDescription,
) -> bool {
	let same_word =
		cached_word.string == new_word.string &&
		cached_word.column == new_word.column &&
		cached_word.row == new_word.row &&
		cached_word.color == new_word.color
	;

	return !same_word;
}

fn update_cached_word(
	new_word_index		: usize,
	new_word			: &WordDescription,
	cached_row			: &mut WordRow,
	to_spawn			: &mut WordsToSpawn,
	to_despawn			: &mut DespawnResource,
) {
	let word_was_despawned = cached_row[new_word_index].entity.is_none();

	if !word_was_despawned {
		let outdated	= &cached_row[new_word_index];
		to_despawn.recursive.push(outdated.entity.unwrap());
	}

	cached_row[new_word_index] = new_word.clone();
	add_word_to_spawn(new_word, to_spawn);
}

fn cleanup_cached_row_from(
	cleanup_index_from	: usize,
	cached_row			: &mut WordRow,
	to_despawn			: &mut DespawnResource
) {
	let cached_row_len	= cached_row.len();
	if cleanup_index_from >= cached_row_len {
		return;
	}

	for i in cleanup_index_from .. cached_row_len {
		let cached_word	= &mut cached_row[i];
		if let Some(entity) = cached_word.entity {
			to_despawn.recursive.push(entity);
			cached_word.entity = None;
		}
	}

	assert!(cleanup_index_from <= cached_row_len);
	cached_row.truncate(cleanup_index_from);
}
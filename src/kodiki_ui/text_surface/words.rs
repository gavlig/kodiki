use bevy :: prelude :: *;

use crate :: {
	kodiki :: DespawnResource,
	bevy_ab_glyph :: { ABGlyphFonts, GlyphWithFonts },
};

use super :: *;

#[derive(Default)]
pub struct ClusterRowState {
	pub word_started	: bool,
	pub ended			: bool,
}

#[derive(Clone, Default, Debug)]
pub struct PathRowColInternal {
	pub path	: Option<std::path::PathBuf>,
	pub row		: Option<usize>,
	pub col		: Option<usize>,

	pub row_internal : usize,
	pub word_chain_start_index : usize,
	pub word_chain_end_index : usize,
}

impl PathRowColInternal {
	pub fn full_path_available(&self) -> bool {
		self.path.is_some() && self.row.is_some() && self.col.is_some()
	}

	pub fn clear(&mut self) {
		*self = Self::default();
	}
}

#[derive(Default)]
pub struct PathRowColParser {
	pub paths	: Vec<PathRowColInternal>,
	pub	current : PathRowColInternal,
		buffer	: String,
}

impl PathRowColParser {
	pub fn on_next_word(
		&mut self,
		next_word : &WordDescription,
		prev_word : Option<&WordDescription>,
	) {
		let Some(prev_word) = prev_word else {
			self.buffer.push_str(next_word.string.as_str());
			self.current.word_chain_start_index = next_word.index;
			return
		};

		let expected_next_word_column = prev_word.column + prev_word.string.len();
		let whitespace_found = expected_next_word_column != next_word.column;

		if whitespace_found {
			self.buffer.clear();
			self.current.word_chain_start_index = next_word.index;
		}

		self.buffer.push_str(next_word.string.as_str());

		// assuming there could be a row and column coming after path
		if self.current.path.is_some() && next_word.is_numeric {
			// since the expected format is very simple:
			// path:row:col
			// where
			// path is String
			// row is usize
			// col is usize
			//
			// we just check if we have row first
			if self.current.row.is_none() {
				if let Ok(row) = next_word.string.parse::<usize>() {
					self.current.row = Some(row);
					self.current.word_chain_end_index = next_word.index;
				}
			// and if row is already Some then assuming it's column
			} else if self.current.col.is_none() {
				if let Ok(col) = next_word.string.parse::<usize>() {
					self.current.col = Some(col);
					self.current.word_chain_end_index = next_word.index;
				}
			}

		}

		if let Some(path) = self.long_string_as_path() {
			if self.current.path.is_some() {
				self.paths.push(self.current.clone());
				self.current.clear();
			}

			self.current.path = Some(path);
			
			// FIXME: we always assume path is on the same row currently
			self.current.row_internal = next_word.row_display;
			
			// cleaning up so that the same path doesnt get caught every next iteration
			self.buffer.clear();
			self.current.word_chain_end_index = next_word.index;
		}
	}

	fn long_string_as_path(&self) -> Option<std::path::PathBuf> {
		let path = std::path::Path::new(self.buffer.as_str());
		if path.is_file() {
			Some(path.into())
		} else {
			None
		}
	}
}

pub fn append_cluster_to_row<'a>(
	cluster				: &impl TextSurfaceCellCluster,
	new_row				: &mut WordsRow,
	new_row_state		: &mut ClusterRowState,
	surface_name		: &String,
	surface_coords		: &TextSurfaceCoords,
	fonts				: &'a ABGlyphFonts<'a>,
) {
	let text			= String::from(cluster.text());

	let text_color		= cluster.foreground();
	let is_space		= text == " " || text == "\t";

	let punctuation_check = |c: char| -> bool { c.is_ascii_punctuation() && c != '_' };
	let numeric_check	= |c: char| -> bool { c.is_numeric() };
	let is_punctuation	= text.chars().all(&punctuation_check);
	let is_numeric		= text.chars().all(&numeric_check);

	let new_glyph_with_fonts = GlyphWithFonts::new(&text, fonts);

	// add new cluster to the last word in new row if it's not space
	if new_row_state.word_started {
		let word		= new_row.last_mut().unwrap();

		let different_color = word.color != text_color;

		let first_glyph_in_word		= String::from(word.string.chars().next().unwrap());
		let first_glyph_with_fonts	= GlyphWithFonts::new(&first_glyph_in_word, fonts);

		let different_font			= first_glyph_with_fonts.current_font() != new_glyph_with_fonts.current_font();
		let is_emoji				= new_glyph_with_fonts.is_emoji;
		let separate_punctuation	= (is_punctuation && !word.is_punctuation) || (!is_punctuation && word.is_punctuation);
		let separate_numeric		= !is_numeric && word.is_numeric;

		let word_ended = is_space || different_color || different_font || is_emoji || separate_punctuation || separate_numeric;

		if !word_ended {
			word.string.push_str(text.as_str());
		} else {
			new_row_state.word_started = false;
		}
	}

	// if new cluster is not space and we haven't started collecting another word start collecting now
	if !is_space && !new_row_state.word_started {
		new_row_state.word_started = true;

		let new_word = WordDescription {
			x				: surface_coords.x,
			y				: surface_coords.y,
			row_display		: surface_coords.row,
			column			: surface_coords.column,
			index			: new_row.len(),
			surface_name	: surface_name.clone(),
			color			: text_color,
			string			: text.clone(),
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
fn word_row_debug_log(row: &WordsRow) -> (String, String) {
	let mut row_strings = String::new();
	let mut row_entities = String::new();
	for (index, word) in row.iter().enumerate() {
		row_strings.push_str(format!("[{}]\"{}\" ", index, word.string).as_str());
		row_entities.push_str(format!("[{}]{:?} ", index, word.entity).as_str());
	}

	(row_entities, row_strings)
}

pub fn update_cached_word(
	new_word	: &WordDescription,
	cached_row	: &mut WordsRow,
	to_spawn	: &mut WordSpawnInfo,
	to_despawn	: &mut DespawnResource,
) {
	let cached_row_len = cached_row.len();
	let new_word_index = new_word.index;

	if new_word_index >= cached_row_len {
		cached_row.push(new_word.clone());
		to_spawn.word_coords.push(WordCoords { row: new_word.row_display, index: new_word_index });
		return;
	}

	let cached_word = &cached_row[new_word_index];
	if word_has_changed(new_word, cached_word) || cached_word.entity.is_none() {
		let word_was_despawned = cached_word.entity.is_none();

		if !word_was_despawned {
			let outdated	= &cached_word;
			to_despawn.recursive.push(outdated.entity.unwrap());
		}

		cached_row[new_word_index] = new_word.clone();
		to_spawn.word_coords.push(WordCoords { row: new_word.row_display, index: new_word_index });
	}
}

pub fn on_cached_row_updated(
	new_row		: &WordsRow,
	cached_row	: &mut WordsRow,
	to_despawn	: &mut DespawnResource,
) {
	let cached_row_len = cached_row.len();

	let new_row_len = new_row.len();
    if new_row_len == 0 || new_row_len < cached_row_len {
		cleanup_cached_row_from(new_row_len, cached_row, to_despawn);
	}
}

fn word_has_changed(
	new_word	: &WordDescription,
	cached_word	: &WordDescription,
) -> bool {
	let same_word =
		cached_word.string == new_word.string &&
		cached_word.column == new_word.column &&
		cached_word.row_display == new_word.row_display &&
		cached_word.color == new_word.color
	;

	return !same_word;
}

fn cleanup_cached_row_from(
	cleanup_index_from	: usize,
	cached_row			: &mut WordsRow,
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
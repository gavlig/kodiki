use bevy :: prelude :: *;
use bevy :: utils :: HashMap;

use std :: ops :: Range;

use super :: { SyncDataDoc, SyncDataString, VersionType };

use helix_core :: regex :: RegexBuilder;
use helix_view :: editor :: SearchConfig;
use helix_view :: Document;

pub type MatchRange = Range<usize>;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum SearchKind {
	Common,
	Selection
}

#[derive(Resource)]
pub struct MatchesMapCache {
	pub map			: HashMap<SearchKind, Matches>
}

impl Default for MatchesMapCache {
	fn default() -> Self {
		let mut map = HashMap::new();
		map.insert(SearchKind::Common, Matches::default());
		map.insert(SearchKind::Selection, Matches::default());

		Self { map }
	}
}

#[derive(Default)]
pub struct Matches {
	pub cache		: Option<SyncDataString>,
	pub vec			: Vec<MatchRange>,
	pub version		: VersionType,
}

impl Matches {
	pub fn is_empty(&self) -> bool {
		self.cache.is_none()
	}

	pub fn clear(&mut self) {
		if self.cache.is_some() {
			self.version += 1;
		}
		self.cache = None;
		self.vec.clear();
	}

	fn find(
		pattern: &str,
		doc: &Document,
		search_config: &SearchConfig,
		ignore_case: bool,
	) -> Vec<MatchRange>{
		let mut matches = Vec::new();

		if pattern.len() < 2 {
			return matches
		}
		
		let mut case_insensitive = ignore_case;

		// cheat: wrap special character in [] so that regex doesnt freak out trying to make sense of the pattern while we are actually looking for a substring match
		let mut pattern_modified = String::new();
		for char in pattern.chars() {
			let is_punctuation = char.is_ascii_punctuation();
			if is_punctuation {
				pattern_modified.push('[');
			}

			pattern_modified.push(char);

			if is_punctuation {
				pattern_modified.push(']');
			}

			if search_config.smart_case && char.is_uppercase() {
				case_insensitive = true;
			}
		}

		let regex = match RegexBuilder::new(pattern_modified.as_str())
			.case_insensitive(case_insensitive)
			.multi_line(false)
			.build()
		{
			Ok(regex)	=> regex,
			Err(_)		=> return Vec::new()
		};

		let text_slice = doc.text().slice(..);
		let text = text_slice.to_string();

		for regex_match in regex.find_iter(&text) {
			let match_range	= regex_match.range();

			let start_char	= text_slice.byte_to_char(match_range.start);
			let end_char	= text_slice.byte_to_char(match_range.end);

			if end_char == 0 {
				// skip empty matches that don't make sense
				continue;
			}

			matches.push(start_char..end_char);
		}

		matches
	}

	pub fn cache_outdated(&self, doc: &Document, theme: &str) -> bool {
		if let Some(cache) = &self.cache {
			cache.doc.outdated(doc, theme)
		} else {
			true
		}
	}

	pub fn update_required(&self, new_pattern: &String, doc: &Document, theme: &str) -> bool {
		if let Some(cache) = &self.cache {
			&cache.string != new_pattern || cache.doc.outdated(doc, theme)
		} else {
			true
		}
	}

	pub fn update(
		&mut self,
		pattern			: &str,
		doc				: &Document,
		theme			: &str,
		search_config	: &SearchConfig,
		ignore_case		: bool,
	) {
		if pattern.len() == 0 {
			return
		}

		let new_matches = Matches::find(pattern, doc, search_config, ignore_case);

		self.vec		= new_matches;
		self.version	+= 1;

		self.cache = Some(SyncDataString {
			doc : SyncDataDoc {
				id		: doc.id(),
				version : doc.version(),
				theme	: theme.into(),
				..default()
			},
			string : pattern.into()
		});
	}
}
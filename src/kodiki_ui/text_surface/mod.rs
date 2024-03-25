use bevy :: prelude :: *;

use bitflags :: bitflags;

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines :: { * };

use crate :: {
	z_order,
	bevy_ab_glyph	:: ABGlyphFonts,
	kodiki			:: DespawnResource,
	kodiki_ui		:: { * , text_surface  :: words :: PathRowColParser , raypick :: * }
};

mod words;
mod coloring_lines;

pub mod systems;

pub use words :: { ClusterRowState, PathRowColInternal };
pub use coloring_lines::RowState as ColoringLineRowState;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextSurfacePlacement {
    #[default]
    Unknown,

    Top,
    Center,
    Bottom,
    Left,
    Right,

    TopLeft,
    TopRight,

    CenterLeft,
    CenterRight,

    BottomLeft,
    BottomRight,

    AreaCoordinates,
}

bitflags! {
    #[derive(Default, Clone, Copy, Eq, PartialEq, Debug)]
    pub struct TextSurfaceAnchor : u32 {
        const Unknown = 0b000;
        const Top = 0b001;
        const Bottom = 0b010;
        const Fixed = 0b100;
    }
}

pub struct TextSurfaceFlags {
    pub anchor: TextSurfaceAnchor,
    pub placement: TextSurfacePlacement,
}

impl Default for TextSurfaceFlags {
    fn default() -> Self {
        Self {
            anchor: TextSurfaceAnchor::Top,
            placement: TextSurfacePlacement::Center,
        }
    }
}

impl TextSurfaceFlags {
    pub fn editor() -> Self {
        Self {
            anchor: TextSurfaceAnchor::Top,
            placement: TextSurfacePlacement::Center,
        }
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct WordDescription {
	pub string			: String,
	pub color			: Color,
	pub row_display		: usize,
	pub column			: usize,
	pub index			: usize,
	pub x				: f32,
	pub y				: f32,
	pub entity			: Option<Entity>,
	pub mesh_entity		: Option<Entity>,

	pub surface_name	: String,
	pub is_punctuation	: bool,
	pub is_numeric		: bool,
}

impl Default for WordDescription {
	fn default() -> Self {
		Self {
			string		: String::new(),
			color		: Color::CYAN,
			row_display	: 0,
			column		: 0,
			index		: 0,
			x			: 0.0,
			y			: 0.0,
			entity		: None,
			mesh_entity : None,

			surface_name: String::new(),
			is_punctuation : false,
			is_numeric	: false,
		}
	}
}

impl WordDescription {
	pub fn position(&self) -> Vec3 {
		Vec3::new(self.x, self.y, z_order::surface::text())
	}
}

pub type WordsRow = Vec<WordDescription>;

pub struct WordCoords {
	pub row				: usize,
	pub index			: usize,
}

#[derive(Component, Default)]
pub struct WordSpawnInfo {
	word_coords			: Vec<WordCoords>, // reference to TextSurface::row
	paths				: Vec<PathRowColInternal>,
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct ColoringLineDescription {
	pub color			: Color,
	pub row				: usize,
	pub column			: usize,
	pub line_index		: usize,
	pub x				: f32,
	pub y				: f32,
	pub glyph_width		: f32,
	pub height			: f32,
	pub length			: usize,
	pub entity			: Option<Entity>,

	pub surface_name	: String,
}

impl Default for ColoringLineDescription {
	fn default() -> Self {
		Self {
			color		: Color::CYAN,
			row			: 0,
			column		: 0,
			line_index	: 0,
			x			: 0.0,
			y			: 0.0,
			glyph_width	: 0.0,
			height		: 0.0,
			length		: 0,
			entity		: None,

			surface_name: String::new(),
		}
	}
}

impl ColoringLineDescription {
	pub fn size(&self) -> Vec2 {
		Vec2::new(self.width(), self.height)
	}

	pub fn width(&self) -> f32 {
		self.glyph_width * self.length as f32
	}

	pub fn position(&self) -> Vec3 {
		Vec3::new(
			self.x + (self.width() / 2.0),
			self.y + (self.height / 2.0),
			z_order::surface::coloring()
		)
	}
}

pub type ColoringLineRow = Vec<ColoringLineDescription>;

#[derive(Component, Deref, DerefMut, Default)]
pub struct ColoringLinesToSpawn(Vec<ColoringLineDescription>);

#[derive(Clone, PartialEq, Default, Debug)]
pub struct TextSurfaceRow {
	pub words		: WordsRow,
	pub lines		: ColoringLineRow,
}

impl TextSurfaceRow {
	pub fn clear(&mut self) {
		self.words.clear();
		self.lines.clear();
	}
}

pub type TextSurfaceRows = Vec<TextSurfaceRow>;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct TextSurfaceScrollInfo {
	pub enabled				: bool,
	pub offset				: i32,
}

impl TextSurfaceScrollInfo {
	pub fn offset(&self) -> i32 {
		if self.enabled { self.offset } else { 0 }
	}
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct TextSurfaceCacheInfo {
	pub enabled				: bool,
	pub offset				: i32,
	pub rows_in_viewport		: usize,
}

impl TextSurfaceCacheInfo {
	pub fn offset(&self) -> i32 {
		if self.enabled { self.offset } else { 0 }
	}
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum RowOffsetDirection {
	Up,
	#[default]
	Down
}

impl RowOffsetDirection {
	// defines if vertical axis goes up or down
	pub fn sign(&self) -> f32 {
		match self {
			RowOffsetDirection::Down	=> -1.0,
			RowOffsetDirection::Up		=> 1.0,
		}
	}

	// compensation is added so that symbols in the first row are staying inside the surface bounds if every next row is below of previous
	pub fn compensation(&self) -> f32 {
		match self {
			RowOffsetDirection::Down	=> 1.0,
			RowOffsetDirection::Up		=> 0.0,
		}
	}
}

#[derive(Default, Clone, Copy)]
pub struct TextSurfaceCoords {
	pub x			: f32,
	pub y			: f32,
	pub column		: usize,
	pub row			: usize,

	column_width	: f32,
	row_height		: f32,
	row_offset_sign	: f32,
	cache_offset	: i32,

	row_offset_dir	: RowOffsetDirection
}

impl TextSurfaceCoords {
	pub fn new(
		column_width	: f32,
		row_height		: f32,
		row_offset_dir	: RowOffsetDirection,
	) -> Self {
		let row_offset_sign = row_offset_dir.sign();

		let mut new_surface_coords = Self {
			column_width,
			row_height,
			row_offset_sign,
			row_offset_dir,
			..default()
		};

		// on row 0 we already have an offset so that letters stay inside the surface bounds
		new_surface_coords.calc_glyph_y();

		new_surface_coords
	}

	fn calc_x(&mut self) {
		self.x = self.column as f32 * self.column_width;
	}

	fn calc_y(&mut self) {
		self.y = self.row_height * self.row_offset_sign * self.row as f32
	}

	fn calc_glyph_y(&mut self) {
		self.calc_y();

		// row_offset_compensation is added so that symbols in the first row are staying inside the surface bounds if every next row if below of previous
		let row_offset_compensation = self.row_offset_dir.compensation();

		self.y += self.row_height * self.row_offset_sign * row_offset_compensation
	}

	pub fn next_row(&mut self) {
		self.x		= 0.0;
		self.column	= 0;
		self.row	+= 1;

		self.calc_glyph_y();
	}

	pub fn next_column(&mut self) {
		self.x += self.column_width;
		self.column += 1;
	}

	pub fn calc_coordinates(&mut self) {
		self.calc_x();

		self.calc_y();
	}

	pub fn calc_glyph_coordinates(&mut self) {
		self.calc_x();

		self.calc_glyph_y();
	}
}

pub trait TextSurfaceCellCluster {
	fn text(&self) -> &str;

	fn row(&self) -> usize;
	fn col(&self) -> usize;

	fn foreground(&self) -> Color;
	fn background(&self) -> Color;
}

#[derive(Component, Default)]
pub struct PathRowCol {
	pub file_path	: std::path::PathBuf,
	pub row			: usize,
	pub col			: usize,
	pub entities	: Vec<Entity>,
}

#[derive(Component)]
pub struct TextSurface {
	pub name				: String,

	pub rows				: TextSurfaceRows,

	pub anchor				: TextSurfaceAnchor,
	pub placement			: TextSurfacePlacement,
	pub rows_count			: usize,
	pub columns_count		: usize,
	pub size				: Vec2,

	pub background_entity	: Option<Entity>,
	pub cursor_entities		: Vec<Entity>,

	pub update				: bool,
}

impl Default for TextSurface {
	fn default() -> Self {
		Self {
			name				: String::new(),
			background_entity	: None,
			rows				: TextSurfaceRows::new(),
			anchor				: TextSurfaceAnchor::default(),
			placement			: TextSurfacePlacement::default(),
			rows_count			: 0,
			columns_count		: 0,
			size				: Vec2::ZERO,

			cursor_entities		: Vec::new(),

			update				: true,
		}
	}
}

impl TextSurface {
	pub fn new(
		name			: &str,
		columns_count	: usize,
		anchor			: TextSurfaceAnchor,
		placement		: TextSurfacePlacement,
		background_entity : Option<Entity>,
	) -> TextSurface {

		TextSurface {
			name		: String::from(name),
			columns_count,
			anchor,
			placement,
			background_entity,
			..default()
		}
	}

	pub fn on_resize(
		&mut self,
		new_rows_cnt	: usize,
		new_cols_cnt	: usize,
		despawn			: &mut DespawnResource,
	) {
		self.despawn_unused_rows(new_rows_cnt, despawn);

		self.rows.resize_with(new_rows_cnt, || { TextSurfaceRow::default() });

		self.rows_count = new_rows_cnt;
		self.columns_count = new_cols_cnt;
	}

	fn despawn_unused_rows(
		&mut self,
		new_rows_cnt	: usize,
		despawn			: &mut DespawnResource,
	) {
		let old_rows_cnt = self.rows.len();
		if new_rows_cnt >= old_rows_cnt {
			return;
		}

		for i in new_rows_cnt .. old_rows_cnt {
			self.despawn_row(i, despawn);
		}
	}

	pub fn clear_all_rows(&mut self, despawn: &mut DespawnResource) {
		for i in 0 .. self.rows.len() {
			self.despawn_row(i, despawn);
		}
	}

	fn despawn_row(
		&mut self,
		row_index	: usize,
		despawn		: &mut DespawnResource,
	) {
		let row_len		= self.rows[row_index].words.len();
		for i in 0 .. row_len {
			let spawned_word = &mut self.rows[row_index].words[i];
			if let Some(entity) = spawned_word.entity {
				despawn.recursive.push(entity);
				spawned_word.entity = None;
			}
		}
		self.rows[row_index].words.clear();

		let row_len		= self.rows[row_index].lines.len();
		for i in 0 .. row_len {
			let spawned_line = &mut self.rows[row_index].lines[i];
			if let Some(entity) = spawned_line.entity {
				despawn.recursive.push(entity);
				spawned_line.entity = None;
			}
		}
		self.rows[row_index].lines.clear();
	}

	fn row_as_string(&self, row_index: usize, with_coords: bool) -> String {
		let mut row		= String::from(format!("[{}] ", row_index).as_str());
		let row_len		= self.rows[row_index].words.len();
		for i in 0 .. row_len {
			let word = &self.rows[row_index].words[i];
			if with_coords {
				row.push_str(format!("[{} {}]", word.row_display, word.column).as_str());
			}
			row.push_str(format!("{} ", word.string).as_str());
		}

		row
	}

	pub fn process_cluster_into_row(
		&mut self,
		cluster			: &impl TextSurfaceCellCluster,
		background_color: &Color,
		row_state		: &mut ClusterRowState,
		row				: &mut WordsRow,
	 	row_colors_state: &mut ColoringLineRowState,
	 	row_colors		: &mut ColoringLineRow,
		fonts			: &ABGlyphFonts,
	) {
		let columns_in_page		= self.columns_count;

		let column_width		= fonts.main.horizontal_advance_mono();
		let row_height			= fonts.main.vertical_advance();

		let row_offset_dir = if self.anchor.contains(TextSurfaceAnchor::Bottom) {
			RowOffsetDirection::Up
		} else {
			RowOffsetDirection::Down
		};

		let mut surface_coords 	= TextSurfaceCoords::new(column_width, row_height, row_offset_dir);

		surface_coords.row		= cluster.row();
		surface_coords.column	= cluster.col();
		surface_coords.calc_glyph_coordinates();

		row_state.ended = cluster.col() == columns_in_page - 1;

		words::append_cluster_to_row(
			cluster,
			row,
			row_state,
			&self.name,
			&surface_coords,
			fonts,
		);

		coloring_lines::append_cluster_to_row(
			cluster,
			background_color,
			row_colors,
			row_colors_state,
			&self.name,
			&surface_coords,
			fonts
		);
	}

	pub fn update_cached_row(
		&mut self,
		row_index		: usize,
		new_row			: &WordsRow,
		new_row_colors	: &ColoringLineRow,
		words_to_spawn	: &mut WordSpawnInfo,
		lines_to_spawn	: &mut ColoringLinesToSpawn,
		entities_to_despawn	: &mut DespawnResource,
	) {
		// This can happen when camera changes the amount of visible rows and resize hasn't happened yet
		if row_index >= self.rows.len() {
			return;
		}

		// row of words

		let cached_row = &mut self.rows[row_index].words;

		let mut prev_word : Option<&WordDescription> = None;
		let mut path_parser = PathRowColParser::default();

		for new_word in new_row.iter() {
			words::update_cached_word(
				new_word,
				cached_row,
				words_to_spawn,
				entities_to_despawn
			);

			path_parser.on_next_word(new_word, prev_word);

			prev_word = Some(new_word);
		}

		words_to_spawn.paths.append(&mut path_parser.paths);
		if path_parser.current.path.is_some() {
			words_to_spawn.paths.push(path_parser.current);
		}

		words::on_cached_row_updated(new_row, cached_row, entities_to_despawn);

		// row of coloring lines

		let cached_row = &mut self.rows[row_index].lines;

		coloring_lines::update_cached_row(
			cached_row,
			new_row_colors,
			lines_to_spawn,
			entities_to_despawn
		);
	}
}
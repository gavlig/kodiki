use bevy :: prelude :: *;
use bevy_tweening :: { *, lens :: * };

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines	:: *;

use bevy_reader_camera	:: { TextDescriptor, ReaderCamera };

use crate :: {
	z_order,
	kodiki :: DespawnResource,
	bevy_ab_glyph :: { ABGlyphFont, ABGlyphFonts },
	kodiki_ui :: {
		*,
		utils :: *, color :: *, raypick :: *,
		text_background_quad :: *,
	},
};

use super :: { *, utils :: * };

use helix_core	:: { Selection, RopeSlice };
use helix_term	:: ui :: EditorView;
use helix_tui	:: buffer :: { Buffer as SurfaceHelix, SurfaceAnchor, SurfacePlacement };

use helix_view :: {
	View,
	Theme,
	graphics :: {
		Color as HelixColor,
		Rect as HelixRect,
	},
};

mod words;
mod coloring_lines;

pub mod systems;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct WordDescription {
	pub string			: String,
	pub color			: Color,
	pub row				: usize,
	pub column			: usize,
	pub word_index		: usize,
	pub cached_row_index : usize,
	pub x				: f32,
	pub y				: f32,
	pub entity			: Option<Entity>,
	pub mesh_entity		: Option<Entity>,

	pub surface_name	: String,
	pub is_on_editor	: bool,
	pub is_punctuation	: bool,
	pub is_numeric		: bool,
	pub is_highlighted	: bool, // TODO: probably replace this with a component like with hovered word highlighting
}

impl Default for WordDescription {
	fn default() -> Self {
		Self {
			string		: String::new(),
			color		: Color::CYAN,
			row			: 0,
			column		: 0,
			word_index	: 0,
			cached_row_index : 0,
			x			: 0.0,
			y			: 0.0,
			entity		: None,
			mesh_entity : None,

			surface_name	: String::new(),
			is_on_editor	: false,
			is_punctuation	: false,
			is_numeric		: false,
			is_highlighted	: false,
		}
	}
}

impl WordDescription {
	pub fn position(&self) -> Vec3 {
		Vec3::new(self.x, self.y, z_order::surface::text())
	}
}

pub type WordRow = Vec<WordDescription>;

#[derive(Component, Clone, Debug)]
pub struct WordChildren {
	pub mesh_entity		: Entity,
	pub collision_entity: Entity,
}

#[derive(Resource, Default)]
pub struct WordsToSpawn {
	pub per_surface : HashMap<String, Vec<WordDescription>>
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct ColoringLineDescription {
	pub color			: Color,
	pub row				: usize,
	pub column			: usize,
	pub line_index		: usize,
	pub cached_row_index : usize,
	pub x				: f32,
	pub y				: f32,
	pub glyph_width		: f32,
	pub height			: f32,
	pub length			: usize,
	pub entity			: Option<Entity>,

	pub surface_name	: String,
	pub is_editor		: bool
}

impl Default for ColoringLineDescription {
	fn default() -> Self {
		Self {
			color		: Color::CYAN,
			row			: 0,
			column		: 0,
			line_index	: 0,
			cached_row_index : 0,
			x			: 0.0,
			y			: 0.0,
			glyph_width	: 0.0,
			height		: 0.0,
			length		: 0,
			entity		: None,

			surface_name: String::new(),
			is_editor	: false,
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

#[derive(Resource, Default)]
pub struct ColoringLinesToSpawn {
	pub per_surface : HashMap<String, Vec<ColoringLineDescription>>
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct SurfaceRow {
	pub words		: WordRow,
	pub lines		: ColoringLineRow,
}

impl SurfaceRow {
	pub fn clear(&mut self) {
		self.words.clear();
		self.lines.clear();
	}
}

pub type SurfaceRows = Vec<SurfaceRow>;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct SurfaceBevyScrollInfo {
	pub enabled				: bool,
	pub offset				: i32,
}

impl SurfaceBevyScrollInfo {
	pub fn offset(&self) -> i32 {
		if self.enabled { self.offset } else { 0 }
	}
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct SurfaceBevyCacheInfo {
	pub enabled				: bool,
	pub offset				: i32,
	pub rows_cached			: usize,
	pub rows_in_viewport		: usize,
}

impl SurfaceBevyCacheInfo {
	pub fn offset(&self) -> i32 {
		if self.enabled { self.offset } else { 0 }
	}
}

// representation of helix_tui::buffer::Buffer in Bevy
pub struct SurfaceBevy {
	pub name				: String,
	pub entity  			: Entity,
	pub bg_quad_entity		: Entity,

	pub rows				: SurfaceRows,

	pub anchor				: SurfaceAnchor,
	pub placement			: SurfacePlacement,
	pub area				: HelixRect,
	pub size				: Vec2,

	pub scroll_info			: SurfaceBevyScrollInfo,

	pub diagnostics_highlights : Highlights<SyncDataDiagnostics>,
	pub selection_highlights: Highlights<SyncDataDoc>,
	pub search_highlights	: Highlights<VersionType>,
	pub selection_search_highlights : Highlights<VersionType>,
	pub cursor_highlights	: Highlights<helix_core::Position>,

	pub cursor_entities		: Vec<Entity>,
	pub resizer_entity		: Option<Entity>,

	pub is_editor			: bool,
	pub update				: bool,
}

impl Default for SurfaceBevy {
	fn default() -> Self {
		Self {
			name				: String::new(),
			entity				: Entity::from_raw(0),
			bg_quad_entity		: Entity::from_raw(0),
			rows				: SurfaceRows::new(),
			anchor				: SurfaceAnchor::default(),
			placement			: SurfacePlacement::default(),
			area				: HelixRect::default(),
			size				: Vec2::ZERO,
			scroll_info			: SurfaceBevyScrollInfo::default(),

			diagnostics_highlights		: Highlights::<_>::default(),
			selection_highlights		: Highlights::<_>::default(),
			search_highlights			: Highlights::<_>::default(),
			selection_search_highlights : Highlights::<_>::default(),
			cursor_highlights			: Highlights::<_>::default(),

			cursor_entities		: Vec::new(),
			resizer_entity		: None,

			is_editor			: false,
			update				: true,
		}
	}
}

pub type SurfacesMapBevyInner = HashMap<String, SurfaceBevy>;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct SurfacesMapBevy(SurfacesMapBevyInner);

pub type SurfacesMapHelixInner = HashMap<String, SurfaceHelix>;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct SurfacesMapHelix(SurfacesMapHelixInner);

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
pub struct SurfaceCoords {
	pub x			: f32,
	pub y			: f32,
	pub column		: usize,
	pub row			: usize,

	column_width	: f32,
	row_height		: f32,
	row_offset_sign	: f32,
	scroll_offset	: i32,
	cache_offset	: i32,

	row_offset_dir	: RowOffsetDirection
}

impl SurfaceCoords {
	pub fn new(
		column_width	: f32,
		row_height		: f32,
		row_offset_dir	: RowOffsetDirection,
		scroll_offset	: i32,
	) -> Self {
		let row_offset_sign = row_offset_dir.sign();

		let mut new_surface_coords = Self {
			column_width,
			row_height,
			row_offset_sign,
			scroll_offset,
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
		let row_wscroll	= self.row + self.scroll_offset as usize;
		self.y			= self.row_height * self.row_offset_sign * row_wscroll as f32
	}

	fn calc_glyph_y(&mut self) {
		self.calc_y();

		// row_offset_compensation is added so that symbols in the first row are staying inside the surface bounds if every next row if below of previous
		let row_offset_compensation = self.row_offset_dir.compensation();

		self.y			+= self.row_height * self.row_offset_sign * row_offset_compensation
	}

	pub fn next_row(&mut self) {
		self.x			= 0.0;
		self.column		= 0;
		self.row		+= 1;

		self.calc_glyph_y();
	}

	pub fn next_column(&mut self) {
		self.x += self.column_width;
		self.column += 1;
	}

	pub fn row_index_wcache(&self) -> usize {
		self.row + self.cache_offset as usize
	}

	pub fn row_index_global(&self) -> usize {
		self.row + self.scroll_offset as usize
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

impl SurfaceBevy {
	pub fn spawn(
		name			: &String,
		world_position	: Option<Vec3>,
		editor			: bool,
		scroll_enabled	: bool,
		resizer_entity	: Option<Entity>,
		surface_helix	: &SurfaceHelix,
		font			: &ABGlyphFont,
		mesh_assets		: &mut Assets<Mesh>,
		commands		: &mut Commands
	) -> SurfaceBevy {
		let surface_position = world_position.unwrap_or(Vec3::new(0.0, 0.0, 0.0));

		let surface_entity = commands.spawn((
			VisibilityBundle::default(),
			TransformBundle {
				local : Transform::from_translation(surface_position),
				..default()
			},
			RaypickHover::default()
		)).id();

		let mut surface_bevy = SurfaceBevy {
			entity		: surface_entity,
			name		: name.clone(),
			area		: surface_helix.area,
			is_editor	: editor,
			scroll_info	: SurfaceBevyScrollInfo {
				enabled : scroll_enabled,
				..default()
			},
			resizer_entity,
			..default()
		};

		let is_editor_surface = name == EditorView::ID;

		let fill_vertically = is_editor_surface;
		let quad_in_camera_space = is_editor_surface;
		let top_anchor = surface_helix.anchor == SurfaceAnchor::Top;
		let side_gap = 0.0;

		surface_bevy.bg_quad_entity = TextBackgroundQuad::spawn(
			quad_in_camera_space,
			fill_vertically,
			top_anchor,
			side_gap,
			font,
			mesh_assets,
			commands
		);

		commands.entity(surface_entity).add_child(surface_bevy.bg_quad_entity);

		surface_bevy.update_text_descriptor_from_area(font, commands);

		surface_bevy
	}

	fn update_text_descriptor_from_area(
		&mut self,
		font			: &ABGlyphFont,
		commands		: &mut Commands
	) {
		let columns		= self.area.width as usize;
		let rows		= self.area.height as usize;
		self.update_text_descriptor(columns, rows, font, commands);
	}

	pub fn update_text_descriptor(
		&mut self,
		columns			: usize,
		rows			: usize,
		font			: &ABGlyphFont,
		commands		: &mut Commands
	) {
		let glyph_height = font.vertical_advance();
		let glyph_width	= font.horizontal_advance_mono(); // in monospace font every letter should be of the same width so we pick 'a'

		let text_descriptor = TextDescriptor {
			rows,
			columns,
			glyph_width,
			glyph_height,
		};

		commands.entity(self.entity).insert(text_descriptor);
	}

	fn columns_in_viewport(&self) -> usize {
		self.area.width as usize
	}

	fn rows_in_viewport(&self) -> usize {
		self.area.height as usize
	}

	pub fn update(
		&mut self,
		surface_helix	: &SurfaceHelix,

		scroll_offset	: i32,
		theme			: &Theme,
		fonts			: &ABGlyphFonts,

		words_to_spawn	: &mut WordsToSpawn,
		lines_to_spawn	: &mut ColoringLinesToSpawn,
		despawn			: &mut DespawnResource,

		#[cfg(feature = "debug")]
		lines			: Option<&mut DebugLines>,
	) {
		if !self.update || surface_helix.frozen() {
			return;
		}

		self.anchor		= surface_helix.anchor;
		self.placement	= surface_helix.placement;

		// process scroll to get all offsets correct first before doing any resizing

		if self.scroll_info.enabled && scroll_offset != self.scroll_info.offset {
			self.on_scroll(scroll_offset, despawn);
		}

		// now resize if the amount of rows changed

		self.area.width	= surface_helix.area.width;
		self.area.height= surface_helix.area.height;

		let rows_in_viewport = self.rows_in_viewport();

		if rows_in_viewport as usize != self.rows.len() {
			self.on_resize(rows_in_viewport as usize, despawn);
		}

		let background_style = theme.get("ui.background");
		let background_color = color_from_helix(background_style.bg.unwrap_or(HelixColor::Cyan));

		self.update_rows(
			surface_helix,
			&background_color,
			fonts,
			words_to_spawn,
			lines_to_spawn,
			despawn,

			#[cfg(feature = "debug")]
			lines
		);
	}

	fn on_scroll(
		&mut self,
		scroll_offset	: i32,
		despawn			: &mut DespawnResource
	) {
		self.shift_rows_on_scroll(scroll_offset, despawn);

		self.scroll_info.offset	= scroll_offset;
	}

	fn on_scroll_forced(
		&mut self,
		scroll_offset	: i32
	) {
		self.scroll_info.offset	= scroll_offset;
	}

	fn shift_rows_on_scroll(
		&mut self,
		scroll_offset	: i32,
		despawn			: &mut DespawnResource,
	) {
		let rows_in_viewport		= self.rows_in_viewport();

		let scroll_offset_prev		= self.scroll_info.offset;
		let scroll_offset_delta 	= scroll_offset - scroll_offset_prev;
		let scroll_delta_abs		= scroll_offset_delta.abs() as usize;

		//

		let scroll_down				= scroll_offset_delta > 0;
		let scroll_up				= scroll_offset_delta < 0;

		let overscroll				= scroll_delta_abs >= rows_in_viewport;
		let underscroll				= !scroll_down && !scroll_up;

		//

		let rows_to_despawn			= scroll_delta_abs;
		let rows_to_keep			= rows_in_viewport.saturating_sub(rows_to_despawn);

		#[cfg(feature = "scroll_offset_debug")]
		if scroll_offset_delta != 0 || overscroll {
		 	println!("scroll_offset: {scroll_offset}(from: {scroll_offset_prev}) rows_to_despawn: {rows_to_despawn} rows_to_keep: {rows_to_keep} scroll_offset_delta: {scroll_offset_delta} rows_in_viewport: {rows_in_viewport}");
		}

		if overscroll {
			self.clear_all_rows(despawn);
			return;
		}

		if underscroll {
			return;
		}

		assert!(rows_to_despawn != 0);

		// scrolled down: we despawn the amount of scrolled rows on top and shift the rest up
		if scroll_offset_delta > 0 {
			for i in 0 .. rows_to_despawn {
				self.despawn_row	(i, despawn);
			}

			self.rows.drain(0 .. scroll_delta_abs);
			self.rows.resize_with(rows_in_viewport, || { SurfaceRow::default() });
		// scrolled up: everything is vice versa/inverted. Despawn the amount of scrolled rows on the bottom and shift the rest downwards
		} else if scroll_offset_delta < 0 {
			let despawn_till = rows_in_viewport;
			let despawn_from = rows_in_viewport - rows_to_despawn;

			for i in despawn_from .. despawn_till {
				self.despawn_row	(i, despawn);
			}

			self.rows.truncate(rows_to_keep);

			for _ in 0 .. rows_to_despawn {
				self.rows.insert(0, SurfaceRow::default());
			}
		}
	}

	fn on_resize(
		&mut self,
		new_rows_cnt	: usize,
		despawn			: &mut DespawnResource,
	) {
		self.despawn_unused_rows(new_rows_cnt, despawn);

		self.rows.resize_with(new_rows_cnt, || { SurfaceRow::default() });
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
				row.push_str(format!("[{} {}]", word.row, word.column).as_str());
			}
			row.push_str(format!("{} ", word.string).as_str());
		}

		row
	}

	pub fn update_rows(
		&mut self,
		surface_helix	: &SurfaceHelix,
		background_color: &Color,
		fonts			: &ABGlyphFonts,
		words_to_spawn	: &mut WordsToSpawn,
		lines_to_spawn	: &mut ColoringLinesToSpawn,
		entities_to_despawn	: &mut DespawnResource,

		#[cfg(feature = "debug")]
		lines			: Option<&mut DebugLines>,
	) {
		let rows_in_page			= self.rows_in_viewport();
		let columns_in_page			= self.columns_in_viewport();

		let scroll_offset			= self.scroll_info.offset();

		let cells_helix				= &surface_helix.content;

		let column_width			= fonts.main.horizontal_advance_mono();
		let row_height				= fonts.main.vertical_advance();

		let row_offset_dir = if surface_helix.anchor.contains(SurfaceAnchor::Bottom) {
			RowOffsetDirection::Up
		} else {
			RowOffsetDirection::Down
		};

		let mut surface_coords 		= SurfaceCoords::new(column_width, row_height, row_offset_dir, scroll_offset);

		#[cfg(feature = "debug")]
		let mut debug_coords		= [SurfaceCoords::default(); 4];
		#[cfg(feature = "debug")]
		let mut debug_index			= 0;

		let reverse_range			= row_offset_dir == RowOffsetDirection::Up;
		let row_range				= create_range(0 .. rows_in_page, reverse_range);

		for row_index in row_range {
			let cached_row_index	= surface_coords.row_index_wcache() as usize;

			let mut new_words_state	= words::RowState::default();
			let mut new_words_row	= WordRow::new();

			let mut new_lines_state	= coloring_lines::RowState::default();
			let mut new_lines_row	= ColoringLineRow::new();

			for column_index in 0 .. columns_in_page {
				#[cfg(feature = "debug")]
				if (row_index == 0 || row_index == rows_in_page - 1) && (column_index == 0 || column_index == columns_in_page - 1) {
					debug_coords[debug_index] = surface_coords.clone();
					debug_index += 1;
				}

				let content_index	= (row_index * columns_in_page + column_index) as usize;
				let cell_helix		= &cells_helix[content_index];

				new_words_state.ended = column_index == columns_in_page - 1;
				new_lines_state.ended = new_words_state.ended;

				words::append_symbol(
					&self.name,
					self.is_editor,
					&surface_coords,
					cell_helix,
					&mut new_words_row,
					&mut new_words_state,
					fonts,
				);

				coloring_lines::append_cell(
					&self.name,
					self.is_editor,
					background_color,
					&surface_coords,
					cell_helix,
					&mut new_lines_row,
					&mut new_lines_state,
					fonts
				);

				surface_coords.next_column();
			}

			{
				let cached_words_row = &mut self.rows[cached_row_index].words;
				words::update_cached_row(cached_words_row, &new_words_row, words_to_spawn, entities_to_despawn, &surface_coords);
			}

			{
				let cached_lines_row = &mut self.rows[cached_row_index].lines;
				coloring_lines::update_cached_row(cached_lines_row, &new_lines_row, lines_to_spawn, entities_to_despawn);
			}

			surface_coords.next_row();
		}

		#[cfg(feature = "debug")]
		if let Some(lines) = lines {
			assert!(debug_index == 4, "debug_index: {}", debug_index);
			let start	= Vec3::new(debug_coords[0].x - 1.0, debug_coords[0].y + row_height, z_order::surface::text());
			let end 	= Vec3::new(debug_coords[1].x + 1.0, debug_coords[1].y + row_height, z_order::surface::text());
			lines.line_colored	(start, end, 0.0, Color::LIME_GREEN);

			let start	= Vec3::new(debug_coords[2].x - 1.0, debug_coords[2].y, z_order::surface::text());
			let end 	= Vec3::new(debug_coords[3].x + 1.0, debug_coords[3].y, z_order::surface::text());
			lines.line_colored	(start, end, 0.0, Color::SALMON);
		}
	}

	pub fn animate(
		&self,
		start_position	: Vec3,
		tween_path		: Vec<TweenPoint>,
		commands		: &mut Commands
	) {
		let path_len	= tween_path.len();
		assert!			(path_len > 0);

		let tween_point_first = tween_path.first().unwrap();
		let tween_start = Tween::new(
			tween_point_first.ease_function,
			tween_point_first.delay,
			TransformPositionLens {
				start	: start_position,
				end		: tween_point_first.pos,
			},
		);

		let mut seq		= Sequence::from_single(tween_start);
		for i in 1 .. path_len {
			let tween_point_prev	= &tween_path[i - 1];
			let tween_point			= &tween_path[i];

			let tween	= Tween::new(
				tween_point.ease_function,
				tween_point.delay,
				TransformPositionLens {
					start: tween_point_prev.pos,
					end	: tween_point.pos,
				},
			);

			seq			= seq.then(tween);
		}

		commands.entity(self.entity)
			.insert(Transform::from_translation(start_position))
			.insert(Animator::new(seq))
		;
	}

	// for any surface that is not main/editor
	pub fn calc_attached_position(
		anchor				: SurfaceAnchor,
		placement			: SurfacePlacement,
		area_own			: HelixRect,
		area_parent			: HelixRect,
		reader_camera		: &ReaderCamera,
		camera_transform	: &Transform,
		font				: &ABGlyphFont,
		scroll_offset		: i32,
	) -> Vec3 {
		let row_height		= font.vertical_advance();
		let column_width	= font.horizontal_advance_mono();

		// x and y are at center of screen initially
		let mut	x = camera_transform.translation.x + (-column_width * (area_own.width as f32 / 2.0));
		let mut y = camera_transform.translation.y;
		let mut z = z_order::surface::child_surface();
		let target_pos = match placement {
			SurfacePlacement::Top => {
				y += reader_camera.y_top; // FIXME: surface anchor is not accounted for
				Vec3::new(x, y, z)
			},
			SurfacePlacement::TopRight => {
				y += reader_camera.y_top - row_height * 2.0;
				x += ((area_parent.width as f32 / 2.0) * column_width) - ((area_own.width as f32 / 2.0) * column_width);
				Vec3::new(x, y, z)
			},
			SurfacePlacement::Center => {
				if !anchor.contains(SurfaceAnchor::Fixed) {
					y += row_height * (area_own.height as f32 / 2.0);
				}
				z = z_order::surface::center_surface();

				Vec3::new(x, y, z)
			},
			SurfacePlacement::Bottom => {
				y += reader_camera.y_bottom + row_height; // FIXME: surface anchor is not accounted for
				Vec3::new(x, y, z)
			},
			SurfacePlacement::AreaCoordinates => {
				let row_offset_dir = if anchor.contains(SurfaceAnchor::Bottom) {
					RowOffsetDirection::Up
				} else {
					RowOffsetDirection::Down
				};

				let mut surface_coords	= SurfaceCoords::new(column_width, row_height, row_offset_dir, scroll_offset);
				surface_coords.column	= area_own.x as usize;
				surface_coords.row		= area_own.y as usize;
				surface_coords.calc_coordinates();
				Vec3::new(
					surface_coords.x,
					surface_coords.y,
					z
				)
			}
			_ => panic!(),
		};

		target_pos
	}

	pub fn attached_position(
		&self,
		area_parent			: HelixRect,
		reader_camera		: &ReaderCamera,
		camera_transform	: &Transform,
		font				: &ABGlyphFont
	) -> Vec3 {
		let scroll_offset	= self.scroll_info.offset();

		SurfaceBevy::calc_attached_position(
			self.anchor,
			self.placement,
			self.area,
			area_parent,
			reader_camera,
			camera_transform,
			font,
			scroll_offset,
		)
	}

	fn highlight_entities_mut(&mut self, kind: HighlightKind) -> &mut Vec<Entity> {
		match kind {
			HighlightKind::Diagnostic	=>
				&mut self.diagnostics_highlights.entities,
			HighlightKind::Search		=>
				&mut self.search_highlights.entities,
			HighlightKind::Selection	=>
				&mut self.selection_highlights.entities,
			HighlightKind::SelectionSearch =>
				&mut self.selection_search_highlights.entities,
			HighlightKind::Cursor		=>
				&mut self.cursor_highlights.entities,
		}
	}

	fn highlight_z_offset(&self, kind: HighlightKind) -> f32 {
		match kind {
			HighlightKind::Diagnostic	=>
				z_order::surface::highlight_diagnostic(),
			HighlightKind::Search		=>
				z_order::surface::highlight_search(),
			HighlightKind::Selection	=>
				z_order::surface::highlight_selection(),
			HighlightKind::SelectionSearch =>
				z_order::surface::highlight_search(),
			HighlightKind::Cursor		=>
				z_order::surface::cursor(),
		}
	}

	fn spawn_highlight_precise(
		&mut self,
		start_char			: usize,
		end_char			: usize,
		kind				: HighlightKind,
		view				: &View,
		material_handle		: &Handle<StandardMaterial>,
		doc_slice			: &RopeSlice,
		tab_len				: usize,
		gutter_len			: usize,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		commands			: &mut Commands,
	) {
		let row_height		= fonts.main.vertical_advance();
		let column_width	= fonts.main.horizontal_advance_mono();

		let (highlight_start_line, highlight_end_line) = (doc_slice.char_to_line(start_char), doc_slice.char_to_line(end_char));

		let max_line_len	= view.area.width as usize;
		let horizontal_offset = view.offset_external.horizontal_offset;

		// rendering each selected line separately and accurately
		for line in highlight_start_line ..= highlight_end_line {
			let line_chars		= doc_slice.line(line).len_chars();

			let line_start_char	= doc_slice.line_to_char(line);
			let line_end_char	= line_start_char + line_chars.saturating_sub(1);

			let inner_start_char = line_start_char.max(start_char);
			let inner_end_char = line_end_char.min(end_char);

			// calculate offset before the first character in the line (inner_start_char is a position of first non-whitespace character in the line)
			let mut offset_len = 0;
			for char_index in line_start_char .. inner_start_char {
				if '\t' == doc_slice.char(char_index) {
					offset_len += tab_len - (offset_len % tab_len);
				} else {
					offset_len += 1;
				}
			}

			// since there are tabs that are wider than 1 character we need to calculate width by iterating over every char in the line (probably there will be more than just tab later)
			let mut highlight_len = 0;
			for char_index in inner_start_char .. inner_end_char {
				if '\t' == doc_slice.char(char_index) {
					highlight_len += tab_len - ((offset_len + highlight_len) % tab_len);
				} else {
					highlight_len += 1;
				}
			}

			// cutting of left side of line with horizontal offset
			if horizontal_offset > offset_len {
				highlight_len = highlight_len.saturating_sub(horizontal_offset - offset_len);
				offset_len = 0;
			} else {
				offset_len -= horizontal_offset;
			}

			// cutting off right side of line with max_line_len
			let total_len = gutter_len + offset_len + highlight_len;
			if total_len > max_line_len {
				highlight_len = highlight_len.saturating_sub(total_len - max_line_len);
			}

			if highlight_len == 0 {
				continue;
			}

			let highlight_size	= Vec2::new(highlight_len as f32 * column_width, row_height);
			let highlight_mesh_handle = mesh_assets.add(Rectangle::from_size(highlight_size));

			let highlight_x = (gutter_len as f32 + offset_len as f32 + highlight_len as f32 / 2.0) * column_width;
			let highlight_y	= -((line + 1) as f32 * row_height) + row_height / 2.0; // FIXME: surface anchor is not accounted for. +1 because in editor rows grow downwards and their surface anchor is top

			let highlight_entity = commands.spawn(
				PbrBundle {
					mesh		: highlight_mesh_handle,
					material	: material_handle.clone_weak(),
					transform	: Transform::from_translation(Vec3::new(highlight_x, highlight_y, self.highlight_z_offset(kind))),
					..default()
				}
			).id();

			commands.entity(self.entity).add_child(highlight_entity);

			self.highlight_entities_mut(kind).push(highlight_entity);
		}
	}

	fn spawn_highlight_whole_line(
		&mut self,
		highlight_line		: usize,
		highlight_kind		: HighlightKind,
		highlight_material_handle : &Handle<StandardMaterial>,
		gutter_len			: usize,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		commands			: &mut Commands,
	) {
		let row_height		= fonts.main.vertical_advance();
		let column_width	= fonts.main.horizontal_advance_mono();

		let highlight_len	= self.area.width as usize - gutter_len;
		let highlight_width = highlight_len as f32 * column_width;

		let highlight_size	= Vec2::new(highlight_width, row_height);
		let highlight_mesh_handle = mesh_assets.add(Rectangle::from_size(highlight_size));

		let highlight_x		= (gutter_len as f32 * column_width) + (highlight_width / 2.0);
		let highlight_y		= -((highlight_line + 1) as f32 * row_height) + row_height / 2.0; // FIXME: surface anchor is not accounted for. +1 because in editor rows grow downwards and their surface anchor is top

		let highlight_entity = commands.spawn(
			PbrBundle {
				mesh		: highlight_mesh_handle,
				material	: highlight_material_handle.clone_weak(),
				transform	: Transform::from_translation(Vec3::new(highlight_x, highlight_y, self.highlight_z_offset(highlight_kind) - z_order::half_thickness())),
				..default()
			}
		).id();

		commands.entity(self.entity).add_child(highlight_entity);

		self.highlight_entities_mut(highlight_kind).push(highlight_entity);
	}

	pub fn despawn_highlights(
		&mut self,
		kind				: HighlightKind,
		commands			: &mut Commands,
	) {
		let entities = self.highlight_entities_mut(kind);

		for entity in entities.iter() {
			commands.entity(*entity).despawn_recursive();
		}

		entities.clear();
	}

	pub fn update_selection_highlights(
		&mut self,
		selection			: &Selection,
		doc					: &Document,
		view				: &View,
		theme				: &Theme,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		let selection_scope = theme
			.find_scope_index("ui.selection.primary")
			.or_else(|| theme.find_scope_index("ui.selection"))
			.expect("could not find `ui.selection` scope in the theme!");

		let style = theme.highlight(selection_scope);

		let mut base_color = color_from_helix(style.bg.unwrap_or(HelixColor::Cyan));
		base_color.set_a(0.7);

		let selection_material_handle = get_color_material_walpha_handle(
			base_color,
			AlphaMode::Blend,
			color_materials_cache,
			material_assets
		);

		self.despawn_highlights(HighlightKind::Selection, commands);

		let tab_len = doc.tab_width();

		let mut gutter_len = 0 as usize;
		for gutter_type in view.gutters() {
			gutter_len += gutter_type.width(view, doc);
		}

		let doc_slice = doc.text().slice(..);

		for range in selection.iter() {
			if range.head == range.anchor {
				continue
			}

			let (selection_start_char, selection_end_char) = if range.head > range.anchor {
				(range.anchor, range.head)
			} else {
				(range.head, range.anchor)
			};

			if selection_end_char - selection_start_char == 1 {
				continue
			}

			self.spawn_highlight_precise(
				selection_start_char,
				selection_end_char,
				HighlightKind::Selection,
				view,
				&selection_material_handle,
				&doc_slice,
				tab_len,
				gutter_len,
				fonts,
				mesh_assets,
				commands
			);
		}
	}

	pub fn update_diagnostics_highlights(
		&mut self,
		doc					: &Document,
		view				: &View,
		theme				: &Theme,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		let doc_slice = doc.text().slice(..);

		use helix_core::diagnostic::Severity;
		let get_scope_of = |scope| {
			theme
			.find_scope_index(scope)
			// get one of the themes below as fallback values
			.or_else(|| theme.find_scope_index("diagnostic"))
			.or_else(|| theme.find_scope_index("ui.cursor"))
			.or_else(|| theme.find_scope_index("ui.selection"))
			.expect(
				"at least one of the following scopes must be defined in the theme: `diagnostic`, `ui.cursor`, or `ui.selection`",
			)
		};

		// query the theme color defined in the config
		let hint_scope		= get_scope_of("hint");
		let info_scope		= get_scope_of("info");
		let warning_scope	= get_scope_of("warning");
		let error_scope		= get_scope_of("error");

		type DiagnosticsVec = Vec<std::ops::Range<usize>>;

		let mut info_vec	: DiagnosticsVec = Vec::new();
		let mut hint_vec	= Vec::new();
		let mut warning_vec	= Vec::new();
		let mut error_vec	= Vec::new();

		for diagnostic in doc.diagnostics() {
			// Separate diagnostics into different Vecs by severity.
			let vec = match diagnostic.severity {
				Some(Severity::Info)	=> &mut info_vec,
				Some(Severity::Hint)	=> &mut hint_vec,
				Some(Severity::Warning)	=> &mut warning_vec,
				Some(Severity::Error)	=> &mut error_vec,
				_ => continue,
			};

			// lsp can return a single character diagnostic with start == end which we want to draw as well
			let mut diagnostic_range_end = diagnostic.range.end;
			if diagnostic_range_end == diagnostic.range.start && doc_slice.try_char_to_line(diagnostic_range_end + 1).is_ok() {
				diagnostic_range_end += 1;
			}

			// If any diagnostic overlaps ranges with the prior diagnostic,
			// merge the two together. Otherwise push a new span.
			match vec.last_mut() {
				Some(range) if diagnostic.range.start <= range.end => {
					// This branch merges overlapping diagnostics, assuming that the current
					// diagnostic starts on range.start or later. If this assertion fails,
					// we will discard some part of `diagnostic`. This implies that
					// `doc.diagnostics()` is not sorted by `diagnostic.range`.
					debug_assert!(range.start <= diagnostic.range.start);
					range.end = diagnostic_range_end.max(range.end)
				}
				_ => {
					vec.push(diagnostic.range.start .. diagnostic_range_end)
				},
			}
		}

		self.despawn_highlights(HighlightKind::Diagnostic, commands);

		let mut gutter_len	= 0 as usize;
		for gutter_type in view.gutters() {
			gutter_len		+= gutter_type.width(view, doc);
		}

		let tab_len			= doc.tab_width();

		// lambda to spawn each kind of diagnostics overlay over editor surface
		let mut spawn_diagnostics_vec = |
			diagnostics_vec: &DiagnosticsVec,
			theme_scope: usize,
			alpha_precise: f32,
			alpha_whole: Option<f32>,
		| {

			let style = theme.highlight(theme_scope);

			let mut base_color = color_from_helix(style.fg.unwrap_or(HelixColor::Cyan));
			base_color.set_a(alpha_precise);

			let diag_material_handle = get_color_material_walpha_handle(
				base_color,
				AlphaMode::Blend,
				color_materials_cache,
				material_assets
			);

			for diag in diagnostics_vec.iter() {
				self.spawn_highlight_precise(
					diag.start,
					diag.end,
					HighlightKind::Diagnostic,
					view,
					&diag_material_handle,
					&doc_slice,
					tab_len,
					gutter_len,
					fonts,
					mesh_assets,
					commands
				);

				// a whole line gets spawned for every error on the line and they overlap making color more intense which is kinda cool side effect
				if let Some(alpha_whole) = alpha_whole {
					base_color.set_a(alpha_whole);
					let diag_material_handle = get_color_material_walpha_handle(
						base_color,
						AlphaMode::Blend,
						color_materials_cache,
						material_assets
					);
					let line = doc_slice.char_to_line(diag.start);
					self.spawn_highlight_whole_line(
						line,
						HighlightKind::Diagnostic,
						&diag_material_handle,
						gutter_len,
						fonts,
						mesh_assets,
						commands
					);
				}
			}
		};

		spawn_diagnostics_vec(&info_vec, info_scope, 0.05, None);
		spawn_diagnostics_vec(&hint_vec, hint_scope, 0.05, None);
		spawn_diagnostics_vec(&warning_vec, warning_scope, 0.06, None);
		spawn_diagnostics_vec(&error_vec, error_scope, 0.1, Some(0.02));
	}

	pub fn get_search_highlights_mut(&mut self, kind: SearchKind) -> &mut Highlights<usize> {
		match kind {
			SearchKind::Common => &mut self.search_highlights,
			SearchKind::Selection => &mut self.selection_search_highlights
		}
	}

	pub fn update_search_highlights(
		&mut self,
		matches				: &Vec<MatchRange>,
		search_kind			: SearchKind,
		doc					: &Document,
		view				: &View,
		theme				: &Theme,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		let highlight_kind = HighlightKind::from(search_kind);

		self.despawn_highlights(highlight_kind, commands);

		let search_scope = theme
			.find_scope_index("hint")
			.or_else(|| theme.find_scope_index("info"))
			.or_else(|| theme.find_scope_index("diagnostic"))
			.expect("could not find `hint` scope in the theme!");

		let style = theme.highlight(search_scope);

		let mut base_color = color_from_helix(style.fg.unwrap_or(HelixColor::Cyan));
		base_color.set_a(0.06);

		let search_material_handle = get_color_material_walpha_handle(
			base_color,
			AlphaMode::Blend,
			color_materials_cache,
			material_assets
		);

		let text_slice = doc.text().slice(..);

		let mut gutter_len = 0 as usize;
		for gutter_type in view.gutters() {
			gutter_len 		+= gutter_type.width(view, doc);
		}

		let tab_len			= doc.tab_width();

		for range in matches.iter() {
			self.spawn_highlight_precise(
				range.start,
				range.end,
				highlight_kind,
				view,
				&search_material_handle,
				&text_slice,
				tab_len,
				gutter_len,
				fonts,
				mesh_assets,
				commands
			);
		}
	}

	pub fn update_cursor_highlights(
		&mut self,
		cursor_row			: usize,
		gutter_len			: usize,
		theme				: &Theme,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		self.despawn_highlights(HighlightKind::Cursor, commands);

		let cursor_scope = theme
			.find_scope_index("ui.selection.primary")
			.or_else(|| theme.find_scope_index("ui.selection"))
			.expect("could not find `selection` scope in the theme!");

		let style = theme.highlight(cursor_scope);

		let mut base_color = color_from_helix(style.bg.unwrap_or(HelixColor::Cyan));
		base_color.set_a(0.12);

		let cursor_material_handle = get_color_material_walpha_handle(
			base_color,
			AlphaMode::Blend,
			color_materials_cache,
			material_assets
		);

		self.spawn_highlight_whole_line(
			cursor_row,
			HighlightKind::Cursor,
			&cursor_material_handle,
			gutter_len,
			fonts,
			mesh_assets,
			commands
		);
	}
}
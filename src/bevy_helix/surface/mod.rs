use bevy				:: prelude :: { * };
use bevy				:: utils :: { HashMap };
use bevy_tweening		:: { * };
use bevy_tweening		:: lens :: { * };

use bevy_reader_camera	:: { TextDescriptor, ReaderCamera };

use crate				:: bevy_ab_glyph :: { ABGlyphFont, ABFonts, GlyphMeshesCache, TextMeshesCache, EmojiMaterialsCache };

use super				:: { * };
use super				:: animate :: TweenPoint;

use helix_tui 			:: buffer :: { Buffer as SurfaceHelix, SurfaceAnchor, SurfacePlacement };

use helix_view			:: { Theme };
use helix_view			:: graphics :: { Style };

mod words;
mod quads;

#[derive(Component, Clone, Debug)]
pub struct WordDescription {
	pub string	: String,
	pub row		: u32,
	pub column	: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordBevy {
	pub entity		: Option<Entity>,
	pub string		: String,
	pub color		: helix_view::graphics::Color,
	pub column		: u32,
}

pub type WordRowBevy	= Vec<WordBevy>;
pub type WordRowsBevy	= Vec<WordRowBevy>;

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundQuadBevy {
	pub entity		: Option<Entity>,
	pub color		: helix_view::graphics::Color,
	pub column		: u32,
	pub length		: u32,
}

pub type BackgroundQuadRowBevy = Vec<BackgroundQuadBevy>;
pub type BackgroundQuadRowsBevy	= Vec<BackgroundQuadRowBevy>;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct RowBevy {
	pub words		: WordRowBevy,
	pub quads		: BackgroundQuadRowBevy,
}

impl RowBevy {
	pub fn clear(&mut self) {
		self.words.clear();
		self.quads.clear();
	}
}

pub type RowsBevy = Vec<RowBevy>;

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
	pub rows_cached			: i32,
}

impl SurfaceBevyCacheInfo {
	pub fn offset(&self) -> i32 {
		if self.enabled { self.offset } else { 0 }
	}
}

// representation of helix_tui::buffer::Buffer in Bevy
#[derive(Clone, PartialEq, Debug)]
pub struct SurfaceBevy {
	pub entity  			: Option<Entity>,
	
	pub background_quad_entity	: Option<Entity>,
	pub background_quad_color	: Color,
	
	pub rows				: RowsBevy,
	pub anchor				: SurfaceAnchor,
	pub placement			: SurfacePlacement,
	pub area				: helix_view::graphics::Rect,
	
	pub scroll_info			: SurfaceBevyScrollInfo,
	pub cache_info			: SurfaceBevyCacheInfo,
	
	pub update				: bool,
}

impl Default for SurfaceBevy {
	fn default() -> Self {
		Self {
			entity				: None,
			background_quad_entity	: None,
			background_quad_color	: Color::CYAN,
			rows				: RowsBevy::new(),
			anchor				: SurfaceAnchor::default(),
			placement			: SurfacePlacement::default(),
			area				: helix_view::graphics::Rect::default(),
			scroll_info			: SurfaceBevyScrollInfo::default(),
			cache_info			: SurfaceBevyCacheInfo::default(),
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

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum RowOffsetDirection {
	Up,
	#[default]
	Down
}

#[derive(Default)]
pub struct SurfaceCoords {
	pub x		: f32,
	pub y		: f32,
	pub column	: u32,
	pub row		: u32,
	
	row_offset_sign	: f32,
	row_height		: f32,
	scroll_offset	: i32,
	cache_offset	: i32,
}

impl SurfaceCoords {
	pub fn new(row_offset_dir: RowOffsetDirection, row_height: f32, scroll_offset: i32, cache_offset: i32) -> Self {
		let row_offset_sign = match row_offset_dir {
			RowOffsetDirection::Down	=> -1.0,
			RowOffsetDirection::Up		=> 1.0,
		};
		
		let y = row_height * row_offset_sign * scroll_offset as f32;
		
		Self {
			y,
			row_offset_sign,
			row_height,
			scroll_offset,
			cache_offset,
			..default()
		}
	}
	
	pub fn next_row(&mut self) {
		self.x			= 0.0;
		self.column		= 0;
		self.row		+= 1;
		
		let row_wscroll	= self.row + self.scroll_offset as u32;
		self.y			= self.row_height * self.row_offset_sign * row_wscroll as f32;
	}
	
	pub fn next_column(&mut self, glyph: &String, fonts: &ABFonts) {
		self.x += fonts.main.horizontal_advance(glyph);
		self.column += 1;
	}
	
	pub fn row_wcache(&self) -> u32 {
		self.row + self.cache_offset as u32
	}
}

impl SurfaceBevy {
	pub fn spawn(
		surface_name	: &String,
		world_position	: Option<Vec3>,
		scroll_enabled	: bool,
		cache_enabled	: bool,
		surface_helix	: &SurfaceHelix,
		font			: &ABGlyphFont,
		mesh_assets		: &mut Assets<Mesh>,
		commands		: &mut Commands
	) -> SurfaceBevy
	{
		println!		("spawning surface {}", surface_name);
		
		let surface_position = world_position.unwrap_or(Vec3::new(0.0, 0.0, 0.0));
		let surface_entity = commands.spawn(TransformBundle {
			local		: Transform::from_translation(surface_position),
			..default()
		})
		.insert(VisibilityBundle {
			visibility	: Visibility { is_visible: true },
			..default()
		})
		.id();
		
		let mut surface_bevy = SurfaceBevy::new_with_entity(surface_entity);
		
		surface_bevy.area = surface_helix.area;
		surface_bevy.scroll_info.enabled = scroll_enabled;
		surface_bevy.cache_info.enabled = cache_enabled;
		
		surface_bevy.spawn_surface_quad(surface_name, surface_helix, font, mesh_assets, commands);
		
		surface_bevy.insert_text_descriptor_from_area(font, commands);
	
		surface_bevy
	}
	
	fn new_with_entity(surface_entity: Entity) -> SurfaceBevy {
		SurfaceBevy { entity: Some(surface_entity), ..default() }
	}
	
	fn spawn_surface_quad(
		&mut self,
		surface_name	: &String,
		surface_helix	: &SurfaceHelix,
		font			: &ABGlyphFont,
		mesh_assets		: &mut Assets<Mesh>,
		commands		: &mut Commands
	)
	{
		if let Some(background_entity) = self.background_quad_entity {
			commands.entity(background_entity).despawn();
		}
		
		let row_height	= font.vertical_advance();
		let column_width = font.horizontal_advance_mono();
		let v_down_offset = font.vertical_down_offset();
		
		let width		= surface_helix.area.width;
		let height		= 1; // we use scale to stretch it to camera visibility limits so here it's just 1 row of text
		
		let quad_width	= column_width * width as f32;
		let quad_height	= row_height * height as f32;
		
		let quad_x		= column_width * width as f32 / 2.0;
		let mut quad_y	= -row_height * height as f32 / 2.0;
		// + v_advance because we need to cover row 0 with background quads too
		quad_y 			+= row_height;
		// add offset downwards to cover glyphs with vertical advance (y, g, _ etc)
		quad_y			-= v_down_offset;
		
		println!("spawning surface background quad {} w {} h {} x {:.3} y {:.3}", surface_name, width, height, quad_x, quad_y);
		
		let quad_pos		= Vec3::new(quad_x, quad_y, -font.depth_scaled());
		let quad_entity_id	= 
		spawn::background_quad(
			quad_pos,
			Vec2::new(quad_width, quad_height),
			None,
			mesh_assets,
			commands
		);
		
		self.background_quad_entity = Some(quad_entity_id);
		
		let surface_entity = self.entity.unwrap();
		commands.entity(surface_entity).add_child(quad_entity_id);
	}
	
	fn insert_text_descriptor_from_area(
		&mut self,
		font			: &ABGlyphFont,
		commands		: &mut Commands
	)
	{
		let columns		= self.area.width as u32;
		let rows		= self.area.height as u32;
		self.update_text_descriptor(columns, rows, font, commands);
	}
	
	pub fn update_text_descriptor(
		&mut self,
		columns			: u32,
		rows			: u32,
		font			: &ABGlyphFont,
		commands		: &mut Commands
	)
	{
		let glyph_height = font.vertical_advance();
		let glyph_width	= font.horizontal_advance_mono(); // in monospace font every letter should be of the same width so we pick 'a'
		
		let text_descriptor = TextDescriptor {
			rows,
			columns,
			glyph_width,
			glyph_height,
		};
		
		let surface_entity = self.entity.unwrap();
		commands.entity(surface_entity).insert(text_descriptor);
	}
	
	fn columns_in_page(&self) -> i32 {
		self.area.width as i32
	}
	
	fn rows_in_page(&self) -> i32 {
		self.area.height as i32
	}
	
	fn rows_cache_capacity(&self) -> i32 {
		self.rows_in_page() * 2 // 2 more pages: 1 on top of what came from helix and 1 below to show text when scrolling
	}
	
	fn rows_total(&self) -> i32 {
		self.rows_in_page() + if self.cache_info.enabled { self.rows_cache_capacity() } else { 0 }
	}
	
	pub fn update(
		&mut self,
		surface_helix	: &SurfaceHelix,
		
		row_offset		: i32,
		theme			: &Theme,
		fonts			: &ABFonts,
	
		glyph_meshes_cache	: &mut GlyphMeshesCache,
		text_meshes_cache	: &mut TextMeshesCache,
		helix_colors_cache	: &mut HelixColorsCache,
		emoji_materials_cache : &mut EmojiMaterialsCache,
	
		mesh_assets		: &mut Assets<Mesh>,
		image_assets	: &mut Assets<Image>,
		material_assets	: &mut Assets<StandardMaterial>,
		commands		: &mut Commands,
	)
	{
		if !self.update {
			return;
		}
		
		self.anchor		= surface_helix.anchor;
		self.placement	= surface_helix.placement;
		self.area		= surface_helix.area; // syncing area size first because everything else depends on it
		
		let rows_total	= self.rows_total();
		self.despawn_unused_rows(rows_total as usize, commands);
		self.rows.resize_with	(rows_total as usize, || { RowBevy::default() });
		
		let scroll_offset_prev	= self.scroll_info.offset;
		self.scroll_info.offset	= row_offset;
		
		self.offset_cached_rows(row_offset, scroll_offset_prev, commands);
		
		let background_style = theme.get("ui.background");
		
		self.update_rows(
			surface_helix,
			
			&background_style,
			fonts,
			
			glyph_meshes_cache,
			text_meshes_cache,
			helix_colors_cache,
			emoji_materials_cache,
			
			mesh_assets,
			image_assets,
			material_assets,
			
			commands
		);
		
		self.update_background_quad_color(&background_style, helix_colors_cache, material_assets, commands);
	}

	fn offset_cached_rows(
		&mut self,
		row_offset		: i32,
		row_offset_prev	: i32,
		commands		: &mut Commands,
	)
	{
		if !self.cache_info.enabled {
			return;
		}
		
		let rows_in_page			= self.rows_in_page();
		
		let rows_cache_capacity		= self.rows_cache_capacity();
		let rows_cache_capacity_half = rows_cache_capacity / 2;
		
		let row_offset_delta 		= row_offset - row_offset_prev;
		let _row_offset_delta_clamped = row_offset_delta.clamp(-rows_cache_capacity_half, rows_cache_capacity_half);
		
		let rows_cached				= self.cache_info.rows_cached;
		let cache_offset			= self.cache_info.offset;
		let rows_spawned			= rows_in_page + rows_cached;
		
		self.cache_info.offset		= (self.cache_info.offset + row_offset_delta).clamp(0, rows_cache_capacity as i32);
		self.cache_info.rows_cached	= (rows_cached + row_offset_delta).max(rows_cached).clamp(0, rows_cache_capacity as i32);
		
		//
		
		if row_offset_delta > 0 && (cache_offset + row_offset_delta) > rows_cache_capacity {
			let rows_to_despawn 	= ((rows_cached + row_offset_delta) - rows_cache_capacity).min(rows_spawned);
			let rows_to_offset		= (rows_spawned - rows_to_despawn) as usize;
			
			for i in 0 .. rows_to_offset {
				if i < rows_to_despawn as usize {
					self.despawn_row(i, commands);
				}
				
				let i_offset		= i + rows_to_despawn as usize;
				self.rows[i]		= self.rows[i_offset].clone();
				self.rows[i_offset].clear();
			}
		} else if row_offset_delta < 0 && (cache_offset + row_offset_delta) < 0 {
			let rows_to_despawn = ((cache_offset + row_offset_delta).abs()).min(rows_spawned);
			
			let from	= rows_to_despawn as usize;
			let to		= rows_spawned as usize;
			
			for i in (from .. to).rev() {
				if i >= (rows_spawned - rows_to_despawn) as usize {
					self.despawn_row(i, commands);
				}                     
				
				let i_offset = i - rows_to_despawn as usize;
				self.rows[i] = self.rows[i_offset].clone();
				self.rows[i_offset].clear();
			}
		}
		
		// if row_offset_delta != 0 {
		// 	println!("offset: {row_offset} cached : {rows_cached} delta: {row_offset_delta} clamped: {row_offset_delta_clamped} page: {rows_in_page}");
		// }
	}

	pub fn update_rows(
		&mut self,
		surface_helix	: &SurfaceHelix,
		
		background_style: &Style,
		fonts			: &ABFonts,

		glyph_meshes_cache	: &mut GlyphMeshesCache,
		text_meshes_cache	: &mut TextMeshesCache,
		helix_colors_cache	: &mut HelixColorsCache,
		emoji_materials_cache : &mut EmojiMaterialsCache,
	
		mesh_assets		: &mut Assets<Mesh>,
		image_assets	: &mut Assets<Image>,
		material_assets	: &mut Assets<StandardMaterial>,
		commands		: &mut Commands,
	)
	{
		let rows_in_page			= self.rows_in_page();
		let columns_in_page			= self.columns_in_page();
		
		let scroll_offset			= self.scroll_info.offset();
		let cache_offset			= self.cache_info.offset();
		
		let surface_entity			= self.entity.unwrap();
		let mut surface_children : Vec<Entity> = Vec::new();

		let cells_helix				= &surface_helix.content;
		
		let row_height				= fonts.main.vertical_advance();
		let row_offset_dir			= match surface_helix.anchor {
			SurfaceAnchor::Unknown	=> RowOffsetDirection::Down,
			SurfaceAnchor::Top		=> RowOffsetDirection::Down,
			SurfaceAnchor::Bottom	=> RowOffsetDirection::Up,
		};
		
		let mut surface_coords 		= SurfaceCoords::new(row_offset_dir, row_height, scroll_offset, cache_offset);
		
		let reverse_range			= row_offset_dir == RowOffsetDirection::Up;
		let row_range				= utils::create_range(0 .. rows_in_page, reverse_range);

		for row in row_range {
			let row_cache			= surface_coords.row_wcache();
			
			let mut word_row_state	= words::RowState::default();
			let mut words			= words::Row::new();
			
			let mut quad_row_state	= quads::RowState::default();
			let mut quads			= quads::Row::new();
			
			for column in 0 .. columns_in_page {
				let content_index	= (row * columns_in_page + column) as usize;
				let cell_helix		= &cells_helix[content_index];
				
				{
					
				let words_row_bevy	= &mut self.rows[row_cache as usize].words;
				
				word_row_state.ended = column == columns_in_page - 1;
				quad_row_state.ended = word_row_state.ended;
				
				// if word ended - spawn it, if not ended - add symbol to the word in progress, if space - do nothing
				let mut new_word_entities =
				words::append_symbol(
					&surface_coords,
					words_row_bevy,
					&mut word_row_state,
					
					&mut words,
					cell_helix,
					
					fonts,
					glyph_meshes_cache,
					text_meshes_cache,
					helix_colors_cache,
					emoji_materials_cache,
					
					mesh_assets,
					image_assets,
					material_assets,
					commands
				);
				
				surface_children.append(&mut new_word_entities);
				
				}
				
				{
					
				let quads_row_bevy	= &mut self.rows[row_cache as usize].quads;
				
				let mut new_quad_entities =
				quads::append_quad(
					&background_style,
					&surface_coords,
					quads_row_bevy,
					&mut quad_row_state,
					
					&mut quads,
					cell_helix,
					
					fonts,
					helix_colors_cache,
					
					mesh_assets,
					material_assets,
					commands
				);
				
				surface_children.append(&mut new_quad_entities);
				
				}
				
				surface_coords.next_column(&cell_helix.symbol, fonts);
			}

			surface_coords.next_row();
		}
		
		if surface_children.len() > 0 {
			commands.entity(surface_entity).push_children(surface_children.as_slice());
		}
	}
	
	fn update_background_quad_color(
		&mut self,
		background_style	: &Style,
		helix_colors_cache	: &mut HelixColorsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	)
	{
		if background_style.bg.is_none() {
			return;
		}
		
		let color			= color_from_helix(background_style.bg.unwrap());
		if self.background_quad_color == color {
			return;
		}
		
		self.background_quad_color = color;
		let background_quad_material_handle = get_helix_color_material_handle(
			color,
			helix_colors_cache,
			material_assets
		);

		// replace material to reflect changed color
		if let Some(background_entity) = self.background_quad_entity {
			commands.entity(background_entity).insert(background_quad_material_handle.clone_weak());
		}
	}

	fn despawn_row(
		&mut self,
		row_num			: usize,
		commands		: &mut Commands
	)
	{
		let row_len		= self.rows[row_num].words.len();
		for i in 0 .. row_len {
			let word_bevy = &mut self.rows[row_num].words[i];
			if let Some(entity) = word_bevy.entity {
				commands.entity(entity).despawn_recursive();
				word_bevy.entity = None;
			}
		}
		
		let row_len		= self.rows[row_num].quads.len();
		for i in 0 .. row_len {
			let quad_bevy = &mut self.rows[row_num].quads[i];
			if let Some(entity) = quad_bevy.entity {
				commands.entity(entity).despawn_recursive();
				quad_bevy.entity = None;
			}
		}
	}

	fn despawn_unused_rows(
		&mut self,
		new_rows_cnt	: usize,
		commands		: &mut Commands,
	)
	{
		let old_rows_cnt = self.rows.len();
		if new_rows_cnt < old_rows_cnt {
			for i in new_rows_cnt .. old_rows_cnt {
				self.despawn_row(i, commands);
			}
		}
	}
	
	pub fn animate(
		&self,
		start_position	: Vec3,
		tween_path		: Vec<TweenPoint>,
		commands		: &mut Commands
	)
	{
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
	
		let surface_entity = self.entity.unwrap();
		commands.entity(surface_entity)
			.insert(Transform::from_translation(start_position))
			.insert(Animator::new(seq));
	}
	
	pub fn z_offset(zoom: f32) -> f32 {
		-zoom + 0.05
	}
	
	pub fn symbol_z_offset() -> f32 {
		0.01
	}
	
	pub fn quad_z_offset(font: &ABGlyphFont) -> f32 {
		-font.depth_scaled() + font.depth_scaled() / 5.0
	}
	
	pub fn cursor_z_offset(font: &ABGlyphFont) -> f32 {
		-font.depth_scaled() + font.depth_scaled() / 4.0
	}
	
	pub fn calc_target_position(
		anchor				: SurfaceAnchor,
		placement			: SurfacePlacement,
		area				: helix_view::graphics::Rect,
		reader_camera		: &ReaderCamera,
		camera_transform	: &Transform,
		font				: &ABGlyphFont
	) -> Vec3 {
		let row_height		= font.vertical_advance();
		let column_width	= font.horizontal_advance_mono();
		
		let 	x = camera_transform.translation.x + (-column_width * (area.width as f32 / 2.0));
		let mut y = camera_transform.translation.y;
		let 	z = SurfaceBevy::z_offset();
		let target_pos = match placement {
			SurfacePlacement::Top => {
				y += reader_camera.y_top - row_height;
				Vec3::new(x, y, z)
			},
			SurfacePlacement::Center => {
				let mut y_local = row_height * (area.height as f32 / 2.0);
				if anchor == SurfaceAnchor::Bottom {
					y_local *= -1.0;
				}
				y += y_local;
				Vec3::new(x, y, z)
			},
			SurfacePlacement::Bottom => {
				y += reader_camera.y_bottom + font.vertical_down_offset();
				Vec3::new(x, y, z)
			},
			_ => panic!(),
		};
		
		target_pos
	}
	
	pub fn target_position(
		&self,
		reader_camera		: &ReaderCamera,
		camera_transform	: &Transform,
		font				: &ABGlyphFont
	) -> Vec3 {
		SurfaceBevy::calc_target_position(self.anchor, self.placement, self.area, reader_camera, camera_transform, font)
	}
}
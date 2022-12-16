use bevy				:: prelude :: { * };
use bevy				:: render :: primitives :: { Sphere, Frustum };
use bevy_contrib_colors	:: { Tailwind };

use bevy_reader_camera	:: ReaderCamera;

use crate				:: bevy_ab_glyph::{ UsedFonts, TextMeshesCache };

use super				:: { * };

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };

use helix_view			:: { Theme };
use helix_view::graphics::Color as HelixColor;

mod words;
mod quads;

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

#[derive(Default)]
pub struct TableCoords {
	pub x		: f32,
	pub y		: f32,
	pub column	: u32,
	pub row		: u32,
}

impl TableCoords {
	pub fn next_row(&mut self) {
		self.x			= 0.0;
		self.column		= 0;
		self.row		+= 1;
	}
	
	pub fn next_column(&mut self, glyph: &String, used_fonts: &UsedFonts) {
		self.x += used_fonts.main.horizontal_advance(glyph);
		self.column += 1;
	}
}

pub fn surface(
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,
	
	row_offset		: i32,
	theme			: &Theme,
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
	
	let rows_in_page	= surface_helix.area.height as i32;
	let columns_in_page	= surface_helix.area.width as i32;
	
	let rows_cache_capacity	= rows_in_page * 2; // 2 more pages: 1 on top of what came from helix and 1 below to show text when scrolling
	let rows_cache_capacity_half = rows_cache_capacity / 2;
	
	let rows_total		= rows_in_page + rows_cache_capacity;
	
	despawn_unused_rows	(rows_total as usize, surface_bevy, commands);
	surface_bevy.rows.resize_with(rows_total as usize, || { RowBevy::default() });
	
	let row_offset_prev = surface_bevy.row_offset;
	let row_offset_delta = row_offset - row_offset_prev;
	let row_offset_delta_clamped = row_offset_delta.clamp(-rows_cache_capacity_half, rows_cache_capacity_half);
	
	surface_bevy.row_offset = row_offset;
	
	let rows_cached		= surface_bevy.rows_cached;
	let row_offset_local= surface_bevy.row_offset_local;
	let rows_spawned	= rows_in_page + rows_cached;
	
	surface_bevy.row_offset_local = (surface_bevy.row_offset_local + row_offset_delta).clamp(0, rows_cache_capacity as i32);
	surface_bevy.rows_cached = (surface_bevy.rows_cached + row_offset_delta).max(surface_bevy.rows_cached).clamp(0, rows_cache_capacity as i32);
	
	let mut sss = String::new();
	
	//
	//
	//
	
	if row_offset_delta > 0 && (row_offset_local + row_offset_delta) > rows_cache_capacity {
		sss.push_str("DOWN");
		
		let rows_to_despawn = ((rows_cached + row_offset_delta) - rows_cache_capacity).min(rows_spawned);
		let rows_to_offset	= (rows_spawned - rows_to_despawn) as usize;
		println!("spawned: {rows_spawned} to_offset: {rows_to_offset} to_despawn: {rows_to_despawn}");
		
		for i in 0 .. rows_to_offset {
			if i < rows_to_despawn as usize {
				despawn_row(i, surface_bevy, commands);
			}
			
			let i_offset = i + rows_to_despawn as usize;
			surface_bevy.rows[i] = surface_bevy.rows[i_offset].clone();
			surface_bevy.rows[i_offset].clear();
		}
	} else if row_offset_delta < 0 && (row_offset_local + row_offset_delta) < 0 {
		sss.push_str("UP");
		
		let rows_to_despawn = ((row_offset_local + row_offset_delta).abs()).min(rows_spawned);
		
		let from	= rows_to_despawn as usize;
		let to		= rows_spawned as usize;
		
		println!("from: {from} to: {to} spawned: {rows_spawned} to_despawn: {rows_to_despawn} despawn_until: {}", rows_spawned - rows_to_despawn);
			
		for i in (from .. to).rev() {
			if i >= (rows_spawned - rows_to_despawn) as usize {
				despawn_row(i, surface_bevy, commands);
				println!("despawned {}", i as usize);
			}                     
			
			let i_offset = i - rows_to_despawn as usize;
			surface_bevy.rows[i] = surface_bevy.rows[i_offset].clone();
			surface_bevy.rows[i_offset].clear();
			
			println!("moved {} to {}", i_offset as usize, i);
		}
	} else if row_offset_delta != 0 {
		sss.push_str("HMMM");
	}
	
	if row_offset_delta != 0 {
		println!("{sss} offset: {row_offset} cached : {rows_cached} delta: {row_offset_delta} clamped: {row_offset_delta_clamped} page: {rows_in_page}");
	}
	
	let row_offset_local		= surface_bevy.row_offset_local;
	
	let background_style 		= theme.get("ui.background");
	
	let surface_entity			= surface_bevy.entity.unwrap();
	let mut surface_children : Vec<Entity> = Vec::new();

	let mut table_coords 		= TableCoords::default();
	let v_advance				= used_fonts.main.vertical_advance();
	
	let cells_helix				= &surface_helix.content;

	for row in 0 .. rows_in_page {
		let row_global			= table_coords.row + row_offset as u32;
		let row_local			= table_coords.row + row_offset_local as u32;
		
		table_coords.y			= -v_advance * row_global as f32;
		
		let mut word_row_state	= words::RowState::default();
		let mut words			= words::Row::new();
		
		let mut quad_row_state	= quads::RowState::default();
		let mut quads			= quads::Row::new();
		
		for column in 0 .. columns_in_page {
			let content_index	= (row * columns_in_page + column) as usize;
			let cell_helix		= &cells_helix[content_index];
			
			{
				
			let words_row_bevy	= &mut surface_bevy.rows[row_local as usize].words;
			
			word_row_state.ended = column == columns_in_page - 1;
			quad_row_state.ended = word_row_state.ended;
			
			// if word ended - spawn it, if not ended - add symbol to the word in progress, if space - do nothing
			let mut new_word_entities =
			words::update(
				&table_coords,
				words_row_bevy,
				&mut word_row_state,
				
				&mut words,
				cell_helix,
				
				used_fonts,
				text_meshes_cache,
				helix_colors_cache,
				
				mesh_assets,
				material_assets,
				commands
			);
			
			surface_children.append(&mut new_word_entities);
			
			}
			
			{
				
			let quads_row_bevy	= &mut surface_bevy.rows[row_local as usize].quads;
			
			let mut new_quad_entities =
			quads::update(
				&background_style,
				&table_coords,
				quads_row_bevy,
				&mut quad_row_state,
				
				&mut quads,
				cell_helix,
				
				used_fonts,
				helix_colors_cache,
				
				mesh_assets,
				material_assets,
				commands
			);
			
			surface_children.append(&mut new_quad_entities);
			
			}
			
			table_coords.next_column(&cell_helix.symbol, used_fonts);
		}

		table_coords.next_row();
	}
	
	if surface_children.len() > 0 {
		commands.entity(surface_entity).push_children(surface_children.as_slice());
	}
	
	//
	//
	// Background quad color
	
	if background_style.bg.is_some() {
		let color			= color_from_helix(background_style.bg.unwrap());
		let background_quad_material_handle = get_helix_color_material_handle(
			color,
			helix_colors_cache,
			material_assets
		);

		// replace material to reflect changed color
		if let Some(background_entity) = surface_bevy.background_entity {
			commands.entity		(background_entity)
			.remove::<Handle<StandardMaterial>>()
			.insert(background_quad_material_handle.clone_weak())
			;
		}
	}
}


pub fn cursor(
	cursor			: &mut CursorBevy,
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
}

fn despawn_row(
	row_num			: usize,
	surface_bevy	: &mut SurfaceBevy,
	commands		: &mut Commands
)
{
	let row_len		= surface_bevy.rows[row_num].words.len();
	for i in 0 .. row_len {
		let word_bevy = &mut surface_bevy.rows[row_num].words[i];
		if let Some(entity) = word_bevy.entity {
			commands.entity(entity).despawn_recursive();
			word_bevy.entity = None;
		}
	}
	
	let row_len		= surface_bevy.rows[row_num].quads.len();
	for i in 0 .. row_len {
		let quad_bevy = &mut surface_bevy.rows[row_num].quads[i];
		if let Some(entity) = quad_bevy.entity {
			commands.entity(entity).despawn_recursive();
			quad_bevy.entity = None;
		}
	}
}

fn despawn_unused_rows(
	new_rows_cnt	: usize,
	surface_bevy	: &mut SurfaceBevy,
	commands		: &mut Commands,
) {
	let old_rows_cnt = surface_bevy.rows.len();
	if new_rows_cnt < old_rows_cnt {
		for i in new_rows_cnt .. old_rows_cnt {
			despawn_row(i, surface_bevy, commands);
		}
	}
}
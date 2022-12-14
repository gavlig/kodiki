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

	reader_camera	: &ReaderCamera,
	row_offset_global : i32,
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
	
	// this is only needed for auxiliary surfaces (":" prompt)
	// despawn_unused_rows(surface_helix, surface_bevy, commands);
	
	let rows_helix		= surface_helix.area.height as i32;
	let columns_helix	= surface_helix.area.width as i32;
	
	let rows_scrolling	= rows_helix * 2; // 2 more pages: 1 on top of what came from helix and 1 below to show text when scrolling
	let rows_scrolling_half = rows_scrolling / 2;
	
	let rows_total		= rows_helix + rows_scrolling;
	
	let row_offset_global_cache = surface_bevy.row_offset_global;
	let row_offset_delta = row_offset_global - row_offset_global_cache;
	let row_offset_delta_clamped = row_offset_delta.clamp(-rows_scrolling_half, rows_scrolling_half);
	
	let row_offset_local = (surface_bevy.row_offset_local + row_offset_delta).clamp(0, rows_scrolling_half as i32);
	
	surface_bevy.row_offset_local = row_offset_local;
	surface_bevy.row_offset_global = row_offset_global;
	surface_bevy.rows.resize_with(rows_total as usize, || { RowBevy::default() });
	
	if row_offset_local == rows_scrolling_half && row_offset_delta > 0 {
		let row_offset_delta_clamped = row_offset_delta_clamped as usize; // it is guaranteed to be > 0
		
		for i in 0 .. row_offset_delta_clamped + rows_scrolling_half as usize {
			if i < row_offset_delta_clamped as usize {
				despawn_row(i, surface_bevy, commands);
			}
			
			let i_offset = i + row_offset_delta_clamped;
			surface_bevy.rows[i] = surface_bevy.rows[i_offset].clone();
			surface_bevy.rows[i_offset].clear();
		}
	} else if row_offset_local == 0 && row_offset_delta_clamped < 0 {
		let last_row = rows_total - 1;
		let first_row_to_offset = last_row - rows_scrolling_half + row_offset_delta_clamped;
		
		for i in (first_row_to_offset as usize .. last_row as usize).rev() {
			if i > (last_row + row_offset_delta_clamped) as usize {
				despawn_row(i, surface_bevy, commands);
			}
			
			let i_offset = (i as i32 + row_offset_delta_clamped) as usize;
			surface_bevy.rows[i] = surface_bevy.rows[i_offset].clone();
			surface_bevy.rows[i_offset].clear();
		}
	}
	
	let background_style = theme.get("ui.background");
	
	let surface_entity = surface_bevy.entity.unwrap();
	let mut surface_children : Vec<Entity> = Vec::new();

	let mut table_coords = TableCoords::default();
	let v_advance	= used_fonts.main.vertical_advance();
	
	let cells_helix = &surface_helix.content;

	for row in 0 .. rows_helix {
		let row_with_global_offset	= table_coords.row + row_offset_global as u32;
		let row_with_local_offset	= table_coords.row + row_offset_local as u32;
		
		table_coords.y			= -v_advance * row_with_global_offset as f32;
		
		let mut word_row_state	= words::RowState::default();
		let mut words			= words::Row::new();
		
		let mut quad_row_state	= quads::RowState::default();
		let mut quads			= quads::Row::new();
		
		for column in 0 .. columns_helix {
			let content_index	= (row * columns_helix + column) as usize;
			let cell_helix		= &cells_helix[content_index];
			
			{
				
			let words_row_bevy	= &mut surface_bevy.rows[row_with_local_offset as usize].words;
			
			word_row_state.ended = column == columns_helix - 1;
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
				
			let quads_row_bevy	= &mut surface_bevy.rows[row_with_local_offset as usize].quads;
			
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
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,
	commands		: &mut Commands,
) {
	let old_rows_cnt = surface_bevy.rows.len();
	let new_rows_cnt = surface_helix.area.height as usize;
	if new_rows_cnt < old_rows_cnt {
		for i in new_rows_cnt .. old_rows_cnt {
			despawn_row(i, surface_bevy, commands);
		}
	}
}
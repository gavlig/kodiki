use bevy				:: prelude :: { * };
use bevy				:: render :: primitives :: { Sphere, Frustum };
use bevy_contrib_colors	:: { Tailwind };

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

	row_offset		: u32,
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
	
	despawn_unused_rows(surface_helix, surface_bevy, commands);
	surface_bevy.rows.resize_with(surface_helix.area.height as usize, || { RowBevy::default() });
	
	let background_style = theme.get("ui.background");
	
	let surface_entity = surface_bevy.entity.unwrap();
	let mut surface_children : Vec<Entity> = Vec::new();

	let mut table_coords = TableCoords::default();
	let v_advance	= used_fonts.main.vertical_advance();
	
	let width		= surface_helix.area.width;
	let height		= surface_helix.area.height;
	let cells_helix = &surface_helix.content;

	for row in 0..height {
		table_coords.y = -v_advance * (table_coords.row + row_offset) as f32;
		
		let sphere = Sphere {
			center: Vec3::new(table_coords.x, table_coords.y, 0.0).into(),
			radius: used_fonts.main.vertical_advance(),
		};
		
		if !camera_frustum.intersects_sphere(&sphere, false) {
			continue;
		}
		
		let mut word_row_state	= words::RowState::default();
		let mut words			= words::Row::new();
		
		let mut quad_row_state	= quads::RowState::default();
		let mut quads			= quads::Row::new();
		
		for column in 0..width {
			let content_index	= (row * width + column) as usize;
			let cell_helix		= &cells_helix[content_index];
			
			{
				
			let words_row_bevy	= &mut surface_bevy.rows[row as usize].words;
			
			word_row_state.ended = column == width - 1;
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
				
			let quads_row_bevy	= &mut surface_bevy.rows[row as usize].quads;
			
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
		commands.entity(word_bevy.entity.unwrap()).despawn_recursive();
	}
	
	let row_len		= surface_bevy.rows[row_num].quads.len();
	for i in 0 .. row_len {
		let quad_bevy = &mut surface_bevy.rows[row_num].quads[i];
		commands.entity(quad_bevy.entity.unwrap()).despawn_recursive();
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
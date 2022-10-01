use bevy				:: prelude :: { * };
use bevy_text_mesh		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_fly_camera		:: { * };
use bevy_contrib_colors	:: { Tailwind };

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use ttf2mesh 			:: { Glyph };

use super				:: { * };
use crate				:: game :: DespawnResource;

use	crate::bevy_helix::spawn::mesh as spawn_mesh;
use bevy::render::mesh::shape as render_shape;

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
use helix_view::graphics::Color as HelixColor;

fn calc_vertical_offset(row : f32, reference_glyph : &Glyph) -> f32 {
	row * -0.13 // (reference_glyph.inner.ybounds[0] - reference_glyph.inner.ybounds[1]) / 72.
}

fn color_from_helix(helix_color: HelixColor) -> Color {
	match helix_color {
		HelixColor::Reset => Color::WHITE,
		HelixColor::Black => Color::BLACK,
		HelixColor::Red => Tailwind::RED600,
		HelixColor::Green => Tailwind::GREEN600,
		HelixColor::Yellow => Tailwind::YELLOW600,
		HelixColor::Blue => Tailwind::BLUE600,
		HelixColor::Magenta => Tailwind::PURPLE600,
		HelixColor::Cyan => Color::rgb(0.0, 0.5, 0.5),
		HelixColor::Gray => Tailwind::GRAY600,
		HelixColor::LightRed => Tailwind::RED300,
		HelixColor::LightGreen => Tailwind::GREEN300,
		HelixColor::LightBlue => Tailwind::BLUE300,
		HelixColor::LightYellow => Tailwind::YELLOW300,
		HelixColor::LightMagenta => Tailwind::PURPLE300,
		HelixColor::LightCyan => Color::rgb(0.0, 0.7, 0.7),
		HelixColor::LightGray => Tailwind::GRAY300,
		HelixColor::White => Color::WHITE,
		// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
		HelixColor::Indexed(_i) => { panic!("Indexed color is not supported!"); },// Color::AnsiValue(i), 
		HelixColor::Rgb(r, g, b) => Color::rgb_u8(r, g, b),
	}
}

pub fn surface(
	root_entity		: Entity,
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,
	font_handle 	: &Handle<TextMeshFont>,
	font			: &mut ttf2mesh::TTFFile,
	despawn			: &mut DespawnResource,
	commands		: &mut Commands
)
{
	surface_bevy.content.resize_with(surface_helix.content.len(), || { CellBevy::default() });

	let tab_size	= 4; //.editorconfig
	let font_size	= 9.;
	let font_depth	= 0.1;
	let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5

	let reference_glyph : Glyph = font.glyph_from_char('a').unwrap(); // and omega
	let row_offset = calc_vertical_offset(1.0, &reference_glyph);
	let glyph_width	= reference_glyph.inner.advance * font_size_scalar;
	let glyph_height = row_offset.abs();
	let row_num_offset = 6. * glyph_width;
	let vertical_overlap = 0.05;

	let local_position = Vec3::ZERO;

	let mut children : Vec<Entity> = Vec::new();

	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 3 as u32;
	
	let width = surface_helix.area.width;
	let height = surface_helix.area.height;
	let content_helix = &surface_helix.content;
	let content_bevy = &mut surface_bevy.content;

	for y_cell in 0..height {
		y = calc_vertical_offset(row as f32, &reference_glyph);
		
		for x_cell in 0..width {
			let content_index = (y_cell * width + x_cell) as usize;
			let cell_helix = &content_helix[content_index];
			let cell_bevy = &mut content_bevy[content_index];

			// println!("[{} {}] cell {}", x_cell, y_cell, cell.symbol);

			let column_offset = (column as f32) * glyph_width;
			let x = row_num_offset + column_offset;

			let pos = local_position + Vec3::new(x, y, 0.0);

			let color = color_from_helix(cell_helix.fg);

			if cell_helix.symbol != cell_bevy.symbol {

				if cell_bevy.entity.is_some() {
					despawn.entities.push(cell_bevy.entity.unwrap());
				}

				if cell_helix.symbol != " " {
					let mesh_entity_id =
					spawn_mesh(
						&cell_helix.symbol,
						pos,
						&font_handle,
						SizeUnit::NonStandard(font_size),
						font_depth,
						color,
						commands
					);
					children.push(mesh_entity_id);

					cell_bevy.entity = Some(mesh_entity_id);
				} else {
					cell_bevy.entity = None;
				}

				cell_bevy.symbol = cell_helix.symbol.clone();
			}

			column += 1;
		}

		column		= 0;
		row			+= 1;
	}

	if children.len() > 0 {
		commands.entity(root_entity).push_children(children.as_slice());
	}
}
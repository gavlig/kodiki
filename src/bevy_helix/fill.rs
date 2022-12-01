use bevy				:: prelude :: { * };
use bevy_reader_camera	:: { * };

use super				:: { * };

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };

use crate :: bevy_ab_glyph :: ABGlyphFont;
use crate :: bevy_ab_glyph :: TextMeshesCache;

// assuming surface_bevy was already spawned once
pub fn surface(
	surface_name	: &String,
	surface_bevy	: &mut SurfaceBevy,
	surface_helix	: &SurfaceHelix,
	font			: &ABGlyphFont,
	text_meshes_cache : &mut TextMeshesCache,
	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands
)
{
	let surface_entity = surface_bevy.entity.unwrap();
	
	let new_content_len = surface_helix.content.len();
	let old_content_len = surface_bevy.content.len();
	
	if new_content_len < old_content_len {
		let cleanup_from = new_content_len;
		let cleanup_till = old_content_len;
		
		for i in cleanup_from .. cleanup_till {
			let bg_quad_entity = surface_bevy.content[i].bg_quad_entity.unwrap();
			commands.entity(bg_quad_entity).despawn();
			if let Some(symbol_entity) = surface_bevy.content[i].symbol_entity {
				commands.entity(symbol_entity).despawn();
			}
		}
	}
	
	surface_bevy.content.resize_with(surface_helix.content.len(), || { CellBevy::default() });
	surface_bevy.area = surface_helix.area;

	let v_advance	= font.vertical_advance();
	let h_advance	= font.horizontal_advance(&String::from("a")); // in monospace font every letter should be of the same width so we pick 'a'
	let v_down_offset = font.vertical_down_offset();
	
	let mut children : Vec<Entity> = Vec::new();

	let mut column	= 0 as u32;
	let mut row		= 0 as u32;
	
	let width		= surface_helix.area.width;
	let height		= surface_helix.area.height;
	let content_bevy = &mut surface_bevy.content;
	
	println!("filling surface {} len {} w {} h {}", surface_name, surface_helix.content.len(), width, height);
	
	for y_cell in 0 .. height {
		// -v_advance because we move down with every row
		let mut y 	= -v_advance * y_cell as f32;
		// + v_advance because we need to cover row 0 with background quads too
		y 			+= v_advance;
		// add offset downwards to cover glyphs with vertical advance (y, g, _ etc)
		y			-= v_down_offset;
		
		for x_cell in 0 .. width {
			let content_index = (y_cell * width + x_cell) as usize;

			let cell_bevy	= &mut content_bevy[content_index];
			// there could already be an existing entity
			if cell_bevy.bg_quad_entity.is_none() {

				let column_offset = h_advance * x_cell as f32;
				let x 			= column_offset;
				
				let quad_width	= h_advance;
				let quad_height	= v_advance.abs();
				
				//
				//
				// Background Quad
				
				let quad_pos		= Vec3::new(x, y, -font.depth_scaled());
				let quad_entity_id	= 
				spawn::quad(
					quad_pos,
					Vec2::new(quad_width, quad_height),
					text_meshes_cache,
					mesh_assets,
					commands
				);
				
				cell_bevy.bg_quad_entity = Some(quad_entity_id);

				commands.entity(quad_entity_id)
				.insert(Row { 0: row })
				.insert(Column { 0: column })
				;

				children.push(quad_entity_id);
			}

			column 	+= 1;
		}

		column		= 0;
		row			+= 1;
	}

	//
	//
	//

	let text_descriptor = TextDescriptor {
		rows		: height as u32,
		columns		: width as u32,
		glyph_width	: h_advance,
		glyph_height: v_advance
	};
	commands.entity(surface_entity).insert(text_descriptor);
	
	if children.len() > 0 {
		commands.entity(surface_entity).push_children(children.as_slice());
	}
}

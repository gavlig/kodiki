use bevy				:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_contrib_colors	:: { Tailwind };

use bevy_prototype_debug_lines :: { * };

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use crate				:: bevy_ab_glyph::{ ABGlyphFont, TextMeshesCache };
use crate				:: bevy_ab_glyph :: mesh_generator :: generate_glyph_mesh_wcache;

use super				:: { * };

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
use helix_tui			:: { buffer :: Cell as CellHelix };

use helix_view			:: { Theme };
use helix_view::graphics::Color as HelixColor;

pub fn cursor(
	cursor				: &mut CursorBevy,
	q_cursor_transform	: &mut Query<&mut Transform>,
	
	font				: &ABGlyphFont,
	time				: &Res<Time>,

    app					: &mut NonSendMut<Application>,
)
{
    let editor_area     = app.area;
    let (cursor_pos, cursor_kind) = app.cursor(editor_area);
    if let Some(cursor_pos) = cursor_pos {
        // cursor position changed so we reset easing timer
        if cursor.x != cursor_pos.0
        || cursor.y != cursor_pos.1
        {
            cursor.easing_accum = 0.0;
        }

        cursor.x		= cursor_pos.0;
        cursor.y		= cursor_pos.1;
        cursor.kind		= cursor_kind;
    }

	let v_advance		= font.vertical_advance();
	let h_advance		= font.horizontal_advance(&String::from("a")); // in monospace font every letter should be of the same width so we pick 'a'
	let v_down_offset	= font.vertical_down_offset();

	let glyph_width		= h_advance;
	let glyph_height	= v_advance;

	let cursor_z		= -font.depth_scaled() + (font.depth_scaled() / 4.0);

	// move background quad
	if cursor.entity.is_some() && cursor.easing_accum < 1.0 {
		let column_offset = (cursor.x as f32) * h_advance;
		let row_offset	= (cursor.y as f32) * -v_advance + v_advance; 

		let target_x 	= column_offset	+ (glyph_width / 2.0);
		let target_y 	= row_offset	- (glyph_height / 2.0) - v_down_offset;

		let target_pos	= Vec3::new(target_x, target_y, cursor_z);

		let delta_seconds = time.delta_seconds();
		let delta_accum	= delta_seconds / /*cursor_easing_seconds*/ 0.0001;

		let cursor_entity = cursor.entity.unwrap();
		let mut cursor_transform = q_cursor_transform.get_mut(cursor_entity).unwrap();

		cursor.easing_accum = (cursor.easing_accum + delta_accum).min(1.0);
		cursor_transform.translation = cursor_transform.translation.lerp(target_pos, cursor.easing_accum);
	}
}
use bevy :: { prelude :: * };
use bevy_reader_camera :: { * };

use super:: { * };

pub fn caret_system(
		btn		: Res<Input<MouseButton>>,
		key		: Res<Input<KeyCode>>,
		time	: Res<Time>,
		q_text_descriptor : Query<&TextDescriptor>,
	mut q_caret	: Query<(&mut Transform, &mut Caret, &Parent)>,
) {
	let delta_seconds = time.delta_seconds();

	for (mut transform, mut caret, parent) in q_caret.iter_mut() {
		if key.pressed(KeyCode::Left) {
			caret.column_dec(delta_seconds);
		}

		if key.pressed(KeyCode::Right) {
			caret.column_inc(delta_seconds);
		}

		if key.pressed(KeyCode::Up) {
			caret.row_dec(delta_seconds);
		}

		if key.pressed(KeyCode::Down) {
			caret.row_inc(delta_seconds);
		}

		let column = caret.column as f32;
		let row = caret.row as f32 * -1.0; //if options.invert_y { options.row as f32 } else { -(options.row as f32) };

		let descriptor = q_text_descriptor.get(**parent).unwrap();
		transform.translation.x = column * descriptor.glyph_width;
		transform.translation.y = row * descriptor.glyph_height;
	}
}
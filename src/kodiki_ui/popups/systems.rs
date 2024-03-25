use bevy :: prelude :: *;

use bevy_tweening :: *;

use std :: time :: Duration;

use super :: *;

use crate :: {
	z_order,
	kodiki_ui :: { spawn as spawn_common, color :: *, tween_lens :: * },
	bevy_ab_glyph :: {
		ABGlyphFonts, FontAssetHandles,
		GlyphWithFonts, GlyphMeshesCache, TextMeshesCache, EmojiMaterialsCache,
		glyph_mesh_generator :: generate_string_mesh_wcache, 
		emoji_generator :: { generate_emoji_mesh_wcache, generate_emoji_material_wcache }
	},
};

pub fn spawn_words(
	(
		mut glyph_meshes_cache,
		mut text_meshes_cache,
		mut color_materials_cache,
		mut emoji_materials_cache,

		mut mesh_assets,
		mut image_assets,
		mut material_assets,
			font_assets,
			font_handles,
	)
	:
	(
		ResMut<GlyphMeshesCache>,
		ResMut<TextMeshesCache>,
		ResMut<ColorMaterialsCache>,
		ResMut<EmojiMaterialsCache>,

		ResMut<Assets<Mesh>>,
		ResMut<Assets<Image>>,
		ResMut<Assets<StandardMaterial>>,
		Res<Assets<ABGlyphFont>>,
		Res<FontAssetHandles>,
	),

	mut popups			: ResMut<Popups>,
		q_camera		: Query<(&ReaderCamera, &Transform)>,
	mut commands		: Commands,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);
	
	let column_width	= fonts.main.horizontal_advance_mono();
	let row_height		= fonts.main.vertical_advance();
	
	// let tween_duration	= Duration::from_millis(150);
	// let tween_ease		= EaseFunction::CircularInOut;

	let (reader_camera, camera_transform) = q_camera.single();

	for message in popups.messages.iter() {
		let color = Color::ANTIQUE_WHITE;

		let first_char = message.chars().next().unwrap();
		let first_char_string = String::from(first_char);
		let first_symbol = GlyphWithFonts::new(&first_char_string, &fonts);

		// normal glyphs are made of meshes with simple color-material, emojis are made of simple quad mesh with image-material
		let (string_mesh_handle, string_material_handle) =
		if first_symbol.is_emoji {
			// assert!			(word.string.len() == 1, "for emojis we expect to have 1 word per each emoji! Instead got {} symbols in word [{}]", word.string.len(), word.string);
			(
				generate_emoji_mesh_wcache(&first_symbol, &mut mesh_assets, &mut text_meshes_cache),
				generate_emoji_material_wcache(&first_symbol, &mut image_assets, &mut material_assets, &mut emoji_materials_cache)
			)
		} else {
			(
				generate_string_mesh_wcache(message, first_symbol.current_font(), &mut mesh_assets, &mut glyph_meshes_cache, &mut text_meshes_cache),
				get_color_material_handle(color, &mut color_materials_cache, &mut material_assets)
			)
		};

		// spawning everything we need for word objects
		
		let text_mesh_entity = spawn_common::mesh_material_entity(
			&string_mesh_handle,
			&string_material_handle,
			&mut commands
		);

		let text_collision_entity = spawn_common::string_mesh_collision(
			message,
			fonts.main,
			&mut commands
		);

		let text_position = Vec3::Z * z_order::surface::text();

		let text_entity = commands.spawn((
			TransformBundle {
				local : Transform::from_translation(text_position),
				..default()
			},
			WordSubEntities {
				mesh_entity : text_mesh_entity,
				collision_entity : text_collision_entity
			},
			VisibilityBundle::default(),
			RaypickHover::default(),
		))
		.id();
		
		commands.entity(text_entity)
			.add_child(text_mesh_entity)
			.add_child(text_collision_entity)
		;

		let message_len = message.len();
		let message_width = message_len as f32 * column_width;

		let mut	x = camera_transform.translation.x;
		let mut y = camera_transform.translation.y;
		let 	z = z_order::surface::last();

		x += reader_camera.x_right - message_width;
		y += reader_camera.y_bottom;

		let y_target = y + row_height;
		let y_midair = y_target + row_height / 2.0;
		let x_despawn = x + message_width + 1000.;
		
		let popup_position_spawn = Vec3::new(x, y, z);
		let popup_position_target = Vec3 { y : y_target, ..popup_position_spawn };
		let popup_position_midair = Vec3 { y : y_midair, ..popup_position_target };
		let popup_position_despawn = Vec3 { x : x_despawn, ..popup_position_target };

		let popup_entity = Popups::spawn(
			message.as_str(),
			fonts.main,
			Some(popup_position_spawn),
			&mut mesh_assets,
			&mut commands
		);

		commands.entity(popup_entity).add_child(text_entity);
		
		let tween0 = Tween::new(
			EaseFunction::CircularInOut,
			Duration::from_millis(150),
			TransformLens {
				start	: Transform::from_translation(popup_position_spawn),
				end		: Transform::from_translation(popup_position_target),
			}
		);

		let tween1 = Tween::new(
			EaseFunction::CircularInOut,
			Duration::from_millis(3000),
			TransformLens {
				start	: Transform::from_translation(popup_position_target),
				end		: Transform::from_translation(popup_position_midair),
			}
		)
		.with_repeat_count(2)
		.with_repeat_strategy(RepeatStrategy::MirroredRepeat);
		
		let tween2 = Tween::new(
			EaseFunction::CircularInOut,
			Duration::from_millis(150),
			TransformLens {
				start	: Transform::from_translation(popup_position_target),
				end		: Transform::from_translation(popup_position_despawn),
			}
		);

		let sequence = Sequence::new([tween0, tween1, tween2]);

		// let sequence = tween0.then(tween1).then(tween2);
		
		commands.entity(popup_entity)
			.insert(Animator::new(sequence))
		;
	}
		
	popups.messages.clear();
}

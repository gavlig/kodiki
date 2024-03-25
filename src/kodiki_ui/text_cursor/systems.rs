use bevy :: prelude	:: *;

use super :: *;

use crate :: {
	bevy_ab_glyph :: { FontAssetHandles, ABGlyphFonts },
	bevy_framerate_manager :: { FramerateManager, FramerateMode },
};

pub fn update(
		time				: Res<Time>,
		framerate_manager	: Res<FramerateManager>,
		font_assets			: Res<Assets<ABGlyphFont>>,
		font_handles		: Res<FontAssetHandles>,

	(mut gltf_assets, mut material_assets, mut cursor_asset)
							:
	(ResMut<Assets<Gltf>>, ResMut<Assets<StandardMaterial>>, ResMut<CursorVisualAsset>),

	mut	q_cursor			: Query<(Entity, &mut TextCursor)>,
	mut q_transform			: Query<&mut Transform>,
		q_visibility		: Query<&ViewVisibility>,
) {
	profile_function!();

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let low_fps_mode = framerate_manager.mode() == FramerateMode::Idle;

	for (cursor_entity, mut cursor) in q_cursor.iter_mut() {
		let mut cursor_transform = q_transform.get_mut(cursor_entity).unwrap();

		let Ok(visibility) = q_visibility.get(cursor_entity) else { continue };

		if visibility.get() {
			cursor.update(
				&mut cursor_transform,
				&mut gltf_assets,
				&mut material_assets,
				&mut cursor_asset,
				fonts.main,
				&time,
				low_fps_mode
			);
		}
	}
}
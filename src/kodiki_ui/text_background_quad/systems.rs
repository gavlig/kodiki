use bevy 				:: prelude :: *;

use super :: *;

use crate :: kodiki_ui :: { * , color :: * };
use crate :: bevy_ab_glyph :: FontAssetHandles;

pub fn update_transform(
		q_bg_quad		: Query<(Entity, &TextBackgroundQuad)>,
		q_camera		: Query<(&ReaderCamera, &Transform)>,
	mut	q_transform		: Query<&mut Transform, Without<ReaderCamera>>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
) {
	profile_function!();

	let font			= font_assets.get(&font_handles.main).unwrap();
	let row_height		= font.vertical_advance();
	let column_width	= font.horizontal_advance_mono();

	let (reader_camera, camera_transform) = q_camera.single();

	for (bg_quad_entity, bg_quad) in q_bg_quad.iter() {
		let Ok(mut bg_quad_transform) = q_transform.get_mut(bg_quad_entity) else { continue };

		// if scale of quad is 1 its height == glyph's height and same goes for width
		// so for quad to cover N columns it has to be scaled N times by X axis
		// since we always assume the font is mono, it's horizontal advance is the column width
		//
		// scale_x_multiplier * scale_x_unit == scale

		let mut scale_x_multiplier = bg_quad.columns as f32;
		let 	scale_x_unit = column_width;

		// incorporating side gap into total quad width
		scale_x_multiplier += (bg_quad.side_gap * 2.0) / scale_x_unit;

		let bg_quad_width = scale_x_multiplier * scale_x_unit;

		let 	offset_x = (bg_quad_width / 2.0) - bg_quad.side_gap;
		let mut offset_y;

		// quad follows camera and is always bigger than camera visibility range
		if bg_quad.in_camera_space {
			offset_y = camera_transform.translation.y;
		// world space background quads rely on their size both for scale and position
		} else {
			offset_y = (bg_quad.rows as f32 * row_height) / 2.0;
			if bg_quad.top_anchor {
				offset_y *= -1.0;
			}
		}

		// always fill all vertial space or just fixed amount of rows
		let scale_y = if bg_quad.fill_vertically {
			reader_camera.visible_rows * 2.0
		} else {
			bg_quad.rows as f32
		};

		bg_quad_transform.scale.x = scale_x_multiplier;
		bg_quad_transform.scale.y = scale_y;

		bg_quad_transform.translation.x = offset_x;
		bg_quad_transform.translation.y = offset_y;
	}
}

pub fn update_color(
	mut	q_bg_quad			: Query<(Entity, &mut TextBackgroundQuad)>,
	mut	color_materials_cache : ResMut<ColorMaterialsCache>,
	mut	material_assets		: ResMut<Assets<StandardMaterial>>,
	mut commands			: Commands,
) {
	profile_function!();

	for (bg_quad_entity, mut bg_quad) in q_bg_quad.iter_mut() {
		if bg_quad.color.is_none() || bg_quad.color_internal == bg_quad.color {
			continue;
		}

		// initially all text background quads are spawned invisible to avoid having uninitialized quads being rendered. see Self::spawn_internal	
		if bg_quad.color_internal.is_none() {
			commands.entity(bg_quad_entity).insert((
				Visibility::Inherited,
			));
		}
				
		let background_material_handle = get_color_material_handle(
			bg_quad.color.unwrap(),
			&mut color_materials_cache,
			&mut material_assets
		);

		// replace material to reflect changed color
		commands.entity(bg_quad_entity).insert(background_material_handle.clone_weak());

		bg_quad.color_internal = bg_quad.color;
	}
}

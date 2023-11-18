use bevy :: { prelude :: * , gltf :: Gltf };

use crate :: bevy_ab_glyph :: ABGlyphFont;

pub mod systems;

#[derive(Resource, Default)]
pub struct CursorVisualAsset {
	pub handle: Handle<Gltf>,
	pub loaded: bool
}

#[derive(Component, Debug)]
pub struct TextCursor {
	pub name			: String,
	pub color			: Color,
	pub col				: usize,
	pub row				: usize,
		col_cache		: usize,
		row_cache		: usize,
	pub row_offset_sign	: f32,

	pub easing_accum	: f32,
	pub blink_accum		: f32,
	pub blink_alpha		: f32,
	pub blink_toggle	: bool,
	pub idle_accum		: f32,
}

impl Default for TextCursor {
	fn default() -> Self {
		Self {
			name: "[DefaultTextCursor]".into(),
			color: Color::CYAN,
			col: 0,
			row: 0,
			col_cache: 0,
			row_cache: 0,
			row_offset_sign: -1.0,
			easing_accum: 0.9,
			blink_accum: 0.0,
			blink_alpha: 0.2,
			blink_toggle: false,
			idle_accum: 0.0
		}
	}
}

impl TextCursor {
	pub fn spawn(
		name			: &str,
		cursor_z		: f32,
		font			: &ABGlyphFont,

		gltf_assets		: &mut Assets<Gltf>,
		material_assets	: &mut Assets<StandardMaterial>,
		cursor_asset	: &mut CursorVisualAsset,
		commands		: &mut Commands
	) -> Entity {
		let glyph_width		= font.horizontal_advance_mono();
		let glyph_height	= font.vertical_advance();

		let cursor_scale	= Vec3::new(glyph_width, glyph_height, glyph_height);
		let cursor_pos		= Vec3::new(0., 0., cursor_z);

		// spawn gltf model of cursor and change its material to unlit
		if let Some(cursor_gltf) = gltf_assets.get_mut(&cursor_asset.handle) {
			let cursor_material_handle = cursor_gltf.named_materials["cursor_material"].clone_weak();
			let cursor_material = material_assets.get_mut(&cursor_material_handle).unwrap();

			cursor_material.unlit = true;
			cursor_material.alpha_mode = AlphaMode::Blend;

			let cursor_entity = commands.spawn((
				SceneBundle {
					scene: cursor_gltf.scenes[0].clone(),
					transform: Transform {
						translation: cursor_pos,
						scale: cursor_scale,
						..default()
					},
					..default()
				},
				TextCursor {
					name : String::from(name),
					color : cursor_material.base_color,
					..default()
				}
			)).id();

			cursor_entity
		} else {
			panic!("cursor_asset.handle was not found among gltf_assets! Make sure path to cursor asset glts is correct and asset file exists!");
		}
	}

	pub fn update(
		&mut self,
		cursor_transform	: &mut Transform,
		gltf_assets			: &mut Assets<Gltf>,
		material_assets		: &mut Assets<StandardMaterial>,
		cursor_asset		: &mut CursorVisualAsset,
		font				: &ABGlyphFont,
		time				: &Time,
		low_fps_mode		: bool,
	) {
		// cursor position changed so we reset easing timer
		if self.col_cache != self.col || self.row != self.row_cache {
			self.reset_accum();
		}

		self.col_cache		= self.col;
		self.row_cache		= self.row;

		self.idle_accum		+= time.delta_seconds();

		self.update_transform(cursor_transform, font, time);
		self.update_material(gltf_assets, material_assets, cursor_asset, time, low_fps_mode);
	}

	fn update_transform(
		&mut self,
		cursor_transform	: &mut Transform,
		font				: &ABGlyphFont,
		time				: &Time,
	) {
		let glyph_width		= font.horizontal_advance_mono();
		let glyph_height	= font.vertical_advance();

		let delta_seconds	= time.delta_seconds();

		let column_offset 	= (self.col as f32) * glyph_width;
		let row_offset		= (self.row as f32) * glyph_height * self.row_offset_sign;

		let target_x 		= column_offset	+ (glyph_width / 2.0);
		let target_y 		= row_offset	+ (glyph_height / 2.0) * self.row_offset_sign;

		let target_pos		= Vec3::new(target_x, target_y, cursor_transform.translation.z);

		// move cursor entity until it reaches its target_pos with easing
		if self.easing_accum < 1.0 || !target_pos.abs_diff_eq(cursor_transform.translation, f32::EPSILON) {
			let easing_delta	= delta_seconds / /*cursor_easing_seconds*/ 0.03;
			self.easing_accum	= (self.easing_accum + easing_delta).min(1.0);

			cursor_transform.translation = cursor_transform.translation.lerp(target_pos, self.easing_accum);
		}
	}

	fn update_material(
		&mut self,
		gltf_assets			: &mut Assets<Gltf>,
		material_assets		: &mut Assets<StandardMaterial>,
		cursor_asset		: &mut CursorVisualAsset,

		time				: &Time,
		low_fps_mode		: bool,
	) {
		let delta_seconds	= time.delta_seconds();

		// update gltf material to blink
		if self.idle_accum > 0.5 {
			let blink_delta	= if self.blink_toggle { 1.0 } else { -1.0 } * delta_seconds / /*cursor_blink_seconds*/ 1.3;

			self.blink_accum += blink_delta;

			if self.blink_accum > 1.0 {

				self.blink_accum	= 1.0;
				self.blink_toggle	= false;

			} else if self.blink_accum < 0.0 {

				self.blink_accum	= 0.0;
				self.blink_toggle	= true;

			}
		} else {
			self.blink_accum = 1.0;
		}

		let cursor_alpha = if low_fps_mode {
			if self.blink_toggle { self.blink_alpha } else { 0.0 }
		} else {
			self.blink_accum * self.blink_alpha
		};

		if let Some(cursor_gltf) = gltf_assets.get_mut(&cursor_asset.handle) {
			let cursor_material_handle = cursor_gltf.named_materials["cursor_material"].clone_weak();
			let cursor_material = material_assets.get_mut(&cursor_material_handle).unwrap();

			cursor_material.unlit = true;
			cursor_material.base_color = self.color;
			cursor_material.base_color.set_a(cursor_alpha);
		}
	}

	fn reset_accum(&mut self) {
		self.easing_accum 	= 0.0;
		self.blink_accum	= 0.0;
		self.idle_accum		= 0.0;
	}
}
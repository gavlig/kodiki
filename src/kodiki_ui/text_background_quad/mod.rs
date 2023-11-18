use bevy :: prelude :: *;

use super :: spawn;

use crate :: bevy_ab_glyph :: ABGlyphFont;

pub mod systems;

#[derive(Component, Clone, Copy)]
pub struct TextBackgroundQuad {
	pub color			: Option<Color>,
		color_internal	: Option<Color>,
	pub in_camera_space	: bool,
	pub fill_vertically	: bool,
	pub top_anchor		: bool,
	pub side_gap		: f32,
	pub columns			: usize,
	pub rows			: usize,
}

impl Default for TextBackgroundQuad {
	fn default() -> Self {
        Self {
			color 			: None,
			color_internal	: None,
			in_camera_space	: false,
			fill_vertically	: false,
			top_anchor		: false,
			side_gap		: 0.0,
			columns			: 1,
			rows			: 1,
		}
    }
}

impl TextBackgroundQuad {
	pub fn with_columns(mut self, columns: usize) -> Self {
		self.columns = columns;
		self
	}

	pub fn with_color(mut self, color: Color) -> Self {
		self.color = Some(color);
		self
	}

	pub fn spawn(
		in_camera_space	: bool,
		fill_vertically	: bool,
		top_anchor		: bool,
		side_gap		: f32,
		font			: &ABGlyphFont,
		mesh_assets		: &mut Assets<Mesh>,
		commands		: &mut Commands
	) -> Entity {
		let quad_entity = Self::spawn_internal(
			Vec3::ZERO, /* position is calculated in systems::update_transform */
			true,		/* with_collision */
			None,		/* material_handle is set in update_color system */
			font,
			mesh_assets,
			commands
		);

		commands.entity(quad_entity).insert(
			TextBackgroundQuad {
				in_camera_space,
				fill_vertically,
				top_anchor,
				side_gap,
				..default()
			}
		);

		quad_entity
	}

	pub fn spawn_clone(
		&self,
		font			: &ABGlyphFont,
		mesh_assets		: &mut Assets<Mesh>,
		commands		: &mut Commands
	) -> Entity {
		let quad_entity = Self::spawn_internal(
			Vec3::ZERO, /* position is calculated in systems::update_transform */
			true,		/* with_collision */
			None,		/* material_handle is set in update_color system */
			font,
			mesh_assets,
			commands
		);

		commands.entity(quad_entity).insert(self.clone());

		quad_entity
	}

	fn spawn_internal(
		position		: Vec3,
		with_collision	: bool,
		material_handle	: Option<&Handle<StandardMaterial>>,
		font			: &ABGlyphFont,
		mesh_assets		: &mut Assets<Mesh>,
		commands		: &mut Commands
	) -> Entity {
		// we use scale to stretch surface to the amount of rows/columns it contains so here its size is just 1 symbol
		let row_height	= font.vertical_advance();
		let column_width = font.horizontal_advance_mono();

		let entity = spawn::background_quad(
			position,
			Vec2::new(column_width, row_height),
			with_collision,
			material_handle,
			mesh_assets,
			commands
		);

		// preventing having uninitialized mesh on screen before first update sets it up
		commands.entity(entity).insert(VisibilityBundle { visibility: Visibility::Hidden, ..default() });

		entity
	}
}
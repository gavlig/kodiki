use bevy :: prelude :: *;

use crate :: {
	bevy_ab_glyph :: ABGlyphFont,
	kodiki_ui :: {
		*,
		text_background_quad :: TextBackgroundQuad,
	}
};

pub mod systems;

#[derive(Resource, Default)]
pub struct Popups {
	messages : Vec<String> // NOTE: Copy On Write would be great here
}

#[derive(Component)]
pub struct Popup {
	message : String,
	spawned : bool,
}

impl Popup {
	pub fn new(message : &str) -> Self {
		Popup {
			message : String::from(message),
			spawned : false,
		}
	}
}

impl Popups {
	pub fn add_message(&mut self, message: &String) {
		self.messages.push(message.clone())
	}

	pub fn spawn(
		message			: &str,
		font			: &ABGlyphFont,
		translation		: Option<Vec3>,
		mesh_assets		: &mut Assets<Mesh>,
		commands		: &mut Commands
	) -> Entity {
		let background_quad = TextBackgroundQuad::default()
			.with_columns(message.len())
			.with_color(Color::GRAY)
		;

		let background_quad_entity = background_quad.spawn_clone(font, mesh_assets, commands);

		let popup_entity = commands.spawn((
			Popup::new(message),
			TransformBundle::from_transform(
				Transform::from_translation(translation.unwrap_or(Vec3::ZERO))
			),
			VisibilityBundle::default(),
			RaypickHover::default()
		)).id();

		commands.entity(popup_entity).push_children(&[background_quad_entity]);

		popup_entity
	}
}
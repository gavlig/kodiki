use bevy :: prelude :: *;

pub mod systems;
pub mod systems_utils;

#[derive(Component, Default, Copy, Clone, Debug)]
pub struct RaypickHover {
    hovered: bool,
}

impl RaypickHover {
    pub fn hovered(&self) -> bool {
        self.hovered
    }

	pub fn set_hovered(&mut self, val : bool) {
		self.hovered = val;
	}
}

#[derive(Component)]
pub struct Clicked;

#[derive(Resource)]
pub struct Raypick {
	pub ray_pos		: Vec3,
	pub ray_dir		: Vec3,
	pub ray_dist	: f32,
	pub mouse_pos	: Vec2,

	pub last_hover	: Option<Entity>,
}

impl Default for Raypick {
	fn default() -> Self {
		Self {
			ray_pos		: Vec3::ZERO,
			ray_dir 	: Vec3::NEG_Z,
			ray_dist	: 100.0,
			mouse_pos	: Vec2::ZERO,
			last_hover	: None
		}
	}
}
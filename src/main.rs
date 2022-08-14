#![allow(non_snake_case, dead_code)]

use bevy			:: prelude :: { * };
use bevy_fly_camera	:: { FlyCameraPlugin };

mod game;
use game			:: { GamePlugin };

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)

		.add_plugin(GamePlugin)
		.add_plugin(FlyCameraPlugin)

		.run();
}
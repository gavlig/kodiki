#![allow(non_snake_case, dead_code)]

use bevy			:: prelude :: { * };
use bevy_fly_camera	:: { FlyCameraPlugin };
use bevy_text_mesh	:: prelude :: { * };

mod game;
use game			:: { GamePlugin };

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)

		.add_plugin(GamePlugin)
		.add_plugin(FlyCameraPlugin)
		.add_plugin(TextMeshPlugin)

		.run();
}
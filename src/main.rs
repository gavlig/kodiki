#![allow(non_snake_case, dead_code)]

use bevy			:: prelude :: { * };
use bevy_fly_camera	:: { FlyCameraPlugin };
use bevy_text_mesh	:: prelude :: { * };
use bevy_prototype_lyon :: prelude :: { * };
use bevy_infinite_grid :: { InfiniteGridPlugin };

mod game;
use game			:: { GamePlugin };

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)

		.add_plugin(GamePlugin)
		.add_plugin(FlyCameraPlugin)
		.add_plugin(TextMeshPlugin)
		.add_plugin(InfiniteGridPlugin)

		.run();
}
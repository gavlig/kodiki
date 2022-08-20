#![feature(rustc_private)]
#![allow(non_snake_case, dead_code)]

use bevy			:: prelude :: { * };
use bevy_fly_camera	:: { FlyCameraPlugin };
use bevy_text_mesh	:: prelude :: { * };
use bevy_infinite_grid :: { InfiniteGridPlugin };

use iyes_loopless		:: { prelude :: * };
use bevy_asset_loader	:: { prelude :: * };

mod game;
use game			:: { AppPlugin };

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)

		.add_plugin(AppPlugin)
		.add_plugin(FlyCameraPlugin)
		.add_plugin(TextMeshPlugin)
		// .add_plugin(InfiniteGridPlugin)

		.run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}

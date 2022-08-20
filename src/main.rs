/* 
use bevy::app::AppExit;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;

fn main() {
    App::new()
        .add_loopless_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_collection::<MyAssets>(),
        )
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "images/player.png")]
    single_file: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
*/

#![feature(rustc_private)]
#![allow(non_snake_case, dead_code)]

use bevy			:: prelude :: { * };
use bevy_fly_camera	:: { FlyCameraPlugin };
use bevy_text_mesh	:: prelude :: { * };
use bevy_infinite_grid :: { InfiniteGridPlugin };

use iyes_loopless		:: { prelude :: * };
use bevy_asset_loader	:: { prelude :: * };

use iyes_loopless::prelude::AppLooplessStateExt;
use bevy::asset::LoadState;

mod game;
use game			:: { AppPlugin };

#[derive(AssetCollection)]
struct FontAssets {
    #[asset(path = "images/player.png")]
    single_file: Handle<Image>,
}

// #[derive(AssetCollection)]
// struct FontAssets {
//     #[asset(path = "fonts/droidsans-mono.ttf")]
//     droid_sans_mono: Handle<TextMeshFont>,
// }

fn main() {
	App::new()
		// .add_plugins(DefaultPlugins)

		.add_loopless_state(MyStates::AssetLoading)

		.add_plugins(DefaultPlugins)

		.add_loading_state(
			LoadingState::new(MyStates::AssetLoading)
				.continue_to_state(MyStates::Next)
				.with_collection::<FontAssets>(),
		)

		.add_plugin(AppPlugin)
		// .add_plugin(FlyCameraPlugin)
		// .add_plugin(TextMeshPlugin)
		// .add_plugin(InfiniteGridPlugin)

		.run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}

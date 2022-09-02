#![feature(rustc_private)]
#![allow(non_snake_case, dead_code)]

use bevy			:: { prelude :: *, window :: PresentMode };
use bevy_fly_camera	:: { FlyCameraPlugin };
use bevy_text_mesh	:: prelude :: { * };
// use bevy_infinite_grid :: { InfiniteGridPlugin };
use bevy_shadertoy_wgsl	:: { * };

mod game;
use game			:: { AppPlugin };
mod text;

fn main() {
	App::new()
		.insert_resource(WindowDescriptor {
			width : 1280 as f32,
			height : 720 as f32,
			present_mode : PresentMode::Mailbox,
			scale_factor_override : Some(1.0),
			// decorations: false,
			// mode: WindowMode::SizedFullscreen,
			..default()
		})

		.add_plugins(DefaultPlugins)

		.add_plugin(AppPlugin)
		.add_plugin(FlyCameraPlugin)
		.add_plugin(TextMeshPlugin)
		// .add_plugin(InfiniteGridPlugin)
		.add_plugin(ShadertoyPlugin)

		.run();
}

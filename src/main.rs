#![feature(rustc_private)]
#![allow(non_snake_case, dead_code)]

use bevy			::  prelude :: *;
use bevy :: window 	:: { PresentMode, WindowMode };
use bevy_fly_camera	:: { FlyCameraPlugin };
use bevy_text_mesh	:: prelude :: { * };
use bevy_shadertoy_wgsl	:: { * };
use bevy_debug_text_overlay	:: { OverlayPlugin, screen_print };

// use bevy_infinite_grid :: { InfiniteGridPlugin };

mod game;
use game			:: { AppPlugin };
mod text;
mod bevy_helix;
use bevy_helix      :: { BevyHelixPlugin };

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
        .add_plugin(BevyHelixPlugin)

		.add_plugin(FlyCameraPlugin)
		.add_plugin(TextMeshPlugin)
		.add_plugin(ShadertoyPlugin)
		.add_plugin(OverlayPlugin { font_size: 32.0, fallback_color: Color::rgb(0.8, 0.8, 0.8), ..default() })

		// .add_plugin(InfiniteGridPlugin)

		.add_system(show_fps)

		.run();
}

fn show_fps(time: Res<Time>) {
	let current_time = time.seconds_since_startup();
    let at_interval = |t: f64| current_time % t < time.delta_seconds_f64();
    if at_interval(0.1) {
        let last_fps = 1.0 / time.delta_seconds();
        screen_print!("fps: {last_fps:.0}");
    }
}

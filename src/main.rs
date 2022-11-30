#![feature(rustc_private)]
#![allow(non_snake_case, dead_code)]

use bevy			::  prelude :: *;
use bevy :: window 	:: { PresentMode, WindowMode };
use bevy_fly_camera	:: { FlyCameraPlugin };
use bevy_shadertoy_wgsl	:: { * };
use bevy_mod_picking :: { * };

use bevy_debug_text_overlay	:: { OverlayPlugin, screen_print };
use bevy_polyline	:: { * };
use bevy_prototype_debug_lines :: { * };

// use bevy_infinite_grid :: { InfiniteGridPlugin };

mod game;
use game			:: { AppPlugin };
mod text;
mod bevy_ab_glyph;
use bevy_ab_glyph	:: { ABGlyphPlugin };
mod bevy_helix;
use bevy_helix      :: { BevyHelixPlugin };

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			window: WindowDescriptor {
				width					: 2560.,
				height					: 1440.,
				cursor_visible			: true,
				present_mode			: PresentMode::Mailbox,
				monitor					: MonitorSelection::Current,
				position				: WindowPosition::Centered,
				mode					: WindowMode::Windowed,
				scale_factor_override	: Some(1.0),
				..default()
			},
			..default()
		}))
		.add_plugins(DefaultPickingPlugins)

		.add_plugin(AppPlugin)
		.add_plugin(ABGlyphPlugin)
		.add_plugin(BevyHelixPlugin)

		.add_plugin(FlyCameraPlugin)
		.add_plugin(ShadertoyPlugin)
		.add_plugin(TweeningPlugin)

		.add_plugin(OverlayPlugin { font_size: 16.0, fallback_color: Color::rgb(0.8, 0.8, 0.8), ..default() })
		.add_plugin(PolylinePlugin)
		.add_plugin(DebugLinesPlugin::default())

		// .add_plugin(InfiniteGridPlugin)

		.add_system(show_fps)

		.run();
}

fn show_fps(time: Res<Time>) {
	let current_time = time.elapsed_seconds_f64();
	let at_interval = |t: f64| current_time % t < time.delta_seconds_f64();
	if at_interval(0.1) {
		let last_fps = 1.0 / time.delta_seconds();
		screen_print!("fps: {last_fps:.0}");
	}
}

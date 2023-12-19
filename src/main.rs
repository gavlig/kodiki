#![feature(step_trait)]
#![feature(slice_pattern)] // for Kitty support in WezTerm
#![allow(non_snake_case, dead_code)]

use bevy :: {
	prelude :: *,
	utils	:: Duration,
	log		:: LogPlugin,
	winit	:: WinitSettings,
	window	:: { WindowMode, WindowResolution, Cursor, PresentMode },
};

use bevy_rapier3d :: prelude :: { RapierPhysicsPlugin, RapierDebugRenderPlugin, NoUserData };

use bevy_framepace		:: FramepacePlugin;
use bevy_reader_camera	:: ReaderCameraPlugin;
use bevy_tweening		:: TweeningPlugin;
// use bevy_vfx_bag		:: BevyVfxBagPlugin;

#[cfg(feature = "stats")]
use bevy_debug_text_overlay	:: { OverlayPlugin, screen_print };

#[cfg(feature = "debug")]
use bevy_polyline			:: *; 
#[cfg(feature = "debug")]
use bevy_prototype_debug_lines :: *;

#[cfg(feature = "tracing")]
use bevy_egui				:: { EguiContexts, EguiPlugin };
#[cfg(feature = "tracing")]
use bevy_puffin				:: PuffinTracePlugin;

#[macro_use]
mod macros;

mod z_order;

mod kodiki_ui;
use kodiki_ui				:: KodikiUIPlugin;

mod kodiki;
use kodiki					:: KodikiPlugin;
mod bevy_ab_glyph;
use bevy_ab_glyph			:: ABGlyphPlugin;
mod bevy_helix;
use bevy_helix				:: BevyHelixPlugin;
mod bevy_wezterm;
use bevy_wezterm			:: BevyWezTermPlugin;
mod bevy_framerate_manager;
use bevy_framerate_manager	:: BevyFramerateManagerPlugin;

fn main() {
	let mut app = App::new();

	let mut primary_window = Window {
		resolution	: WindowResolution::new(1920., 1080.),
		position	: WindowPosition::Centered(MonitorSelection::Current),
		mode		: WindowMode::Windowed,
		cursor		: Cursor::default(),
		title		: "Kodiki".into(),
		present_mode: PresentMode::AutoVsync,

		..default()
	};

	primary_window.set_maximized(true);

	app.add_plugins((
		DefaultPlugins
		.set(WindowPlugin {
				primary_window : Some(primary_window),
				close_when_requested : false,
				..default()
			}
		)
		.set(TaskPoolPlugin {
			task_pool_options: TaskPoolOptions::with_num_threads(2),
		})
		.disable::<LogPlugin>(),
	
		FramepacePlugin,
		RapierPhysicsPlugin::<NoUserData>::default(),
		RapierDebugRenderPlugin::default(),

		KodikiUIPlugin,
		KodikiPlugin,
		ABGlyphPlugin,
		BevyFramerateManagerPlugin,
		BevyHelixPlugin,
		BevyWezTermPlugin,

		ReaderCameraPlugin,
		TweeningPlugin,
		// BevyVfxBagPlugin::default()
	))
		

	.insert_resource(WinitSettings {
		focused_mode: bevy::winit::UpdateMode::Continuous,
		unfocused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
			wait: Duration::from_millis(300),
			ignore_cursor_movement: true
		},
		..default()
	});

	#[cfg(feature = "stats")]
	add	.add_plugin(OverlayPlugin { font_size: 16.0, fallback_color: Color::rgb(0.8, 0.8, 0.8), ..default() })
		.add_system(show_fps)
	;

	#[cfg(feature = "debug")]
	app	.add_plugin(PolylinePlugin)
		.add_plugin(DebugLinesPlugin::default())
	;

	#[cfg(feature = "tracing")]
	app	.add_plugin(EguiPlugin)
		.add_plugin(PuffinTracePlugin::new())
		.add_system(show_profiler)
	;

	app.run();
}

#[cfg(feature = "stats")]
fn show_fps(time: Res<Time>) {
	let current_time = time.elapsed_seconds_f64();
	let at_interval = |t: f64| current_time % t < time.delta_seconds_f64();
	if at_interval(0.1) {
		let last_fps = 1.0 / time.delta_seconds();
		screen_print!("fps: {last_fps:.0}");
	}
}

#[cfg(feature = "tracing")]
fn show_profiler(mut contexts: EguiContexts, mut frame_counter: Local<usize>) {
	let ctx = contexts.ctx_mut();
	puffin_egui::profiler_window(ctx);

	*frame_counter += 1;
}
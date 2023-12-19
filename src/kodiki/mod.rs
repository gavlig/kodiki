use bevy :: prelude :: *;
use bevy_tweening :: *;
use bevy_vfx_bag :: post_processing :: masks :: Mask;

pub mod run_condition;
pub mod spawn;
mod systems;
mod systems_util;

use crate :: {
	kodiki_ui :: *,
	bevy_ab_glyph :: *,
	bevy_framerate_manager :: *,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default, States)]
pub enum AppMode {
	#[default]
	AssetLoading,
	AssetsLoaded,
	Main,
	// Unfocused?
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default, States)]
pub enum AppCameraMode {
	#[default]
	Main,			// keyboard input + mouse visible + no camera movement + no zoom available
	Reader,			// keyboard input + mouse invisible + camera slides up/down + zoom available
	Fly,			// no keyboard input + mouse invisible + flying camera + no zoom available (debug)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default, States)]
pub enum AppContext {
	#[default]
	CodeEditor,
	Terminal,
}

#[derive(Component)]
pub enum AppContextSwitcher {
	Entry(AppContext),
}

#[derive(Default, Resource)]
pub struct AppState {
	pub initialized : bool
}

#[derive(Default, Resource)]
pub struct MouseCursorState {
	pub visible : bool
}

#[derive(Default, Resource)]
pub struct CameraIDs {
	pub camera2d: Option<Entity>,
	pub camera3d: Option<Entity>,
}

#[derive(Default, Resource)]
pub struct DespawnResource {
	pub recursive: Vec<Entity>,
	pub children_only: Vec<Entity>,
}

pub struct KodikiPlugin;

impl Plugin for KodikiPlugin {
	fn build(&self, app: &mut App) {
		let clear_color = ClearColor(Color::hex("282c34").unwrap());

		app
			.add_state::<AppMode>()
			.add_state::<AppContext>()
			.add_state::<AppCameraMode>()

			.insert_resource(AppState::default())
			.insert_resource(MouseCursorState::default())

			.insert_resource(clear_color)
			.insert_resource(Msaa::default())
			.insert_resource(CameraIDs::default())

			.insert_resource(DespawnResource::default())

			.configure_sets(
				Update,
				BevyFramerateManagerSystems
				.run_if(run_condition::main_app_mode_no_fly)
			)

			.configure_sets(
				Update,
				KodikiUISystems
				.after(BevyFramerateManagerSystems)
				.run_if(run_condition::main_app_mode_no_fly)
			)

			// asset loading
			.add_systems(Startup, systems::load_assets)
			.add_systems(
				Update,
				(
					systems::font_asset_loading_events,
					systems::gltf_asset_loading_events,
					systems::asset_loading_tracking,
				).run_if(in_state(AppMode::AssetLoading))
			)

			// setup systems, run only once
			.add_systems(
				OnEnter(AppMode::AssetsLoaded),
				(
					systems::setup_world,
				)
			)
			.add_systems(
				OnEnter(AppContext::Terminal),
				(
					systems::spawn_first_terminal,
				)
			)

			// generic app systems
			.add_systems(
				Update,
				(
					systems::kodiki_ui_sync,
					systems::keyboard_input,
					systems::stats,
					systems::process_clicked_terminal_path,
					systems::update_window_title
				).run_if(in_state(AppMode::Main))
			)
			// context switching
			.add_systems(
				Update,
				(
					systems::apply_context_switcher_state,
					systems::highlight_active_context_switcher,
				)
				.chain()
				.run_if(in_state(AppMode::Main))
			)
			// workaround for stuck alt when switching desktop with ctrl+alt+X
			// .add_system(systems::on_window_unfocused.in_set(OnUpdate(AppCameraMode::Fly)))
			// bevy_tweening animator systems
			.add_systems(
				Update,
				(
					asset_animator_system::<StandardMaterial>,
					component_animator_system::<Mask>,
				).in_set(AnimationSystem::AnimationUpdate)
			)
			// unified despawning through a resource
			.add_systems(PostUpdate, systems::despawn)
 		;
	}
}

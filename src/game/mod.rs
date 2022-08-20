use bevy				:: { prelude :: *, window :: PresentMode };
use bevy_mod_picking	:: { * };
use iyes_loopless		:: { prelude :: * };
use bevy_asset_loader	:: { prelude :: * };
use bevy_text_mesh		:: { prelude :: * };

use std					:: { path::PathBuf };
use serde				:: { Deserialize, Serialize };

pub mod spawn;
mod systems;
use systems				:: *;

#[derive(Component, Default, Clone)]
pub struct LogHolder {
	pub data : String
}

pub struct SpawnArguments<'a0, 'a1, 'b0, 'b1, 'c, 'd, 'e> {
	pub meshes				: &'a0 mut ResMut<'a1, Assets<Mesh>>,
	pub materials			: &'b0 mut ResMut<'b1, Assets<StandardMaterial>>,
	pub commands			: &'c mut Commands<'d, 'e>
}

#[derive(Debug, Clone, Copy)]
pub struct RespawnableEntity {
	pub entity	: Entity,
	pub respawn	: bool
}

impl Default for RespawnableEntity {
	fn default() -> Self {
		Self {
			  entity		: Entity::from_bits(0)
			, respawn		: false
		}
	}
}

#[derive(Default)]
pub struct DespawnResource {
	pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppMode {
	AssetLoading,
	AssetsLoaded,
	Main,
    Editor,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
	fn build(&self, app: &mut App) {
		let clear_color = ClearColor(Color::hex("282c34").unwrap());

        app	
			// .add_loopless_state(AppMode::AssetLoading)

			// .add_loading_state(
			// 	LoadingState::new(AppMode::AssetLoading)
			// 		.continue_to_state(AppMode::AssetsLoaded)
			// 		.with_collection::<FontAssets>(),
			// )

			// .add_system_set(
			// 	ConditionSet::new()
			// 		.run_in_state(MyStates::Next)
			// 		.with_system(setup_world_system)
			// 		.into(),
			// )

			// .add_plugin		(PickingPlugin)
			// .add_plugin		(InteractablePickingPlugin)
			// .add_plugins	(HighlightablePickingPlugins)

			.insert_resource(clear_color)
			
			.insert_resource(Msaa			::default())
			.insert_resource(DespawnResource::default())

			.insert_resource(WindowDescriptor { present_mode : PresentMode::Mailbox, ..default() })
			
 			// .add_startup_system(setup_lighting_system)
 			// .add_startup_system(setup_world_system)
 			// .add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)

			// .add_enter_system(AppMode::AssetsLoaded, setup_world_system.run_if_resource_equals(AppMode::AssetsLoaded))
			// .add_system_set	(SystemSet::on_enter(AppMode::MainMode).with_system(setup_world_system))

			// input
			.add_system		(cursor_visibility_system)
			.add_system		(input_misc_system)

			.add_system_to_stage(CoreStage::PostUpdate, despawn_system)
 			;
	}
}

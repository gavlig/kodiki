use bevy				:: { prelude :: *, window :: PresentMode };
use bevy_mod_picking	:: { * };
use iyes_loopless		:: { prelude :: * };

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
pub enum GameMode {
    Editor,
    InGame,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		let clear_color = ClearColor(
			Color::rgb(
				0xF9 as f32 / 255.0,
				0xF9 as f32 / 255.0,
				0xFF as f32 / 255.0,
			));

        app	.add_loopless_state(GameMode::Editor)

			.add_plugin		(PickingPlugin)
			.add_plugin		(InteractablePickingPlugin)

			.insert_resource(clear_color)
			
			.insert_resource(Msaa			::default())
			.insert_resource(DespawnResource::default())

			.insert_resource(WindowDescriptor { present_mode : PresentMode::Mailbox, ..default() })
			
 			.add_startup_system(setup_lighting_system)
 			.add_startup_system(setup_world_system)
 			.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)

			// input
			.add_system		(cursor_visibility_system)
			.add_system		(input_misc_system)

			.add_system_to_stage(CoreStage::PostUpdate, despawn_system)
 			;
	}
}


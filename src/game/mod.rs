use bevy				:: { prelude :: * };
use bevy_mod_picking	:: { * };
use iyes_loopless		:: { prelude :: * };
use bevy_shadertoy_wgsl :: { * };

pub mod spawn;
mod systems;
use systems				:: *;
mod utils;

use crate				:: text;

use crate				:: bevy_ab_glyph :: { * };

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

#[derive(Default)]
pub struct MouseCursorState {
	pub visible : bool
}

#[derive(Default)]
pub struct FontAssetHandles {
	pub droid_sans_mono: Handle<ABGlyphFont>,
	pub open_dyslexic: Handle<ABGlyphFont>,
	pub source_code_pro: Handle<ABGlyphFont>,
	pub B612: Handle<ABGlyphFont>,
	pub share_tech: Handle<ABGlyphFont>,

	pub ubuntu_mono: Handle<ABGlyphFont>,

	pub loaded_cnt: u32,
}

#[derive(Default)]
pub struct CameraIDs {
	pub camera2d: Option<Entity>,
	pub camera3d: Option<Entity>,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
	fn build(&self, app: &mut App) {
		let clear_color = ClearColor(Color::hex("282c34").unwrap());

		let w = 1280;
    	let h = 720;

        app
			.add_plugin		(PickingPlugin)
			.add_plugin		(InteractablePickingPlugin)
			.add_plugins	(HighlightablePickingPlugins)

			.add_loopless_state(AppMode::AssetLoading)

			.insert_resource(MouseCursorState::default())
			.insert_resource(FontAssetHandles::default())

			.insert_resource(clear_color)
			.insert_resource(Msaa::default())
			.insert_resource(CameraIDs::default())

			.insert_resource(ShadertoyCanvas {
				width:		w,
				height:		h,
				borders:	0.0,
				position:	Vec3::new(0.0, 0.0, 0.0),
				active:		true,
			})

			.add_startup_system(setup_shadertoy)
			.add_startup_system(load_assets)

			.add_system		(asset_loading_events.run_in_state(AppMode::AssetLoading))
			
			.add_enter_system_set(
				AppMode::AssetsLoaded,
				SystemSet::new()
				.with_system(setup_world_system)
				.with_system(setup_lighting_system)
				.with_system(setup_camera_system)
			)

			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Main)
				.with_system(input_system)
				.with_system(cursor_visibility_system)
				.with_system(stats_system)
				.with_system(text::caret_system)
				.into()
			)

			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Editor)
				.with_system(input_system)
				.with_system(cursor_visibility_system)
				.into()
			)

			.add_system_to_stage(CoreStage::PostUpdate, despawn_system)
			.insert_resource(DespawnResource::default())
 			;
	}
}

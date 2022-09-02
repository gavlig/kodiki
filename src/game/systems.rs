use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy			:: core_pipeline :: clear_color :: ClearColorConfig;
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use iyes_loopless	:: { prelude :: * };
use bevy_shadertoy_wgsl :: { * };

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

use super           :: { * };

use crate			:: text;

pub fn setup_world_system(
	mut	meshes		: ResMut<Assets<Mesh>>,
	mut	materials	: ResMut<Assets<StandardMaterial>>,
		font_handles: Res<FontAssetHandles>,
	mut fonts		: ResMut<Assets<TextMeshFont>>,
	mut camera_ids	: ResMut<CameraIDs>,
		ass			: Res<AssetServer>,
	mut commands	: Commands,
) {
	spawn::camera	(&mut camera_ids, &mut commands);

	// spawn::infinite_grid(&mut commands);

	// spawn::world_axis	(&mut meshes, &mut materials, &mut commands);

	spawn::fixed_sphere	(Transform::identity(), 0.02, Color::SEA_GREEN, &mut meshes, &mut materials, &mut commands);

	// without font we can't go further
	let mut font		= fonts.get_mut(&font_handles.droid_sans_mono).unwrap();
	
	let mut pos			= Vec3::new(0.0, 0.0, 0.0);
	text::spawn::file(
		"playground/herringbone_spawn.rs", // rustc_ast.rs",
		&font_handles.droid_sans_mono,
		&mut font.ttf_font,
		pos,
		&mut meshes,
		&mut materials,
		&mut commands
	);

	// pos.x				+= 10.0;
	// spawn::file_text	(
	// 	"playground/rapier_parallel_solver_constraints.rs",
	// 	&font_handles.droid_sans_mono,
	// 	&mut font.ttf_font,
	// 	pos,
	// 	&mut commands
	// );

	// pos.x				+= 10.0;
	// spawn::file_text	(
	// 	"playground/rustc_ast.rs",
	// 	&font_handles.droid_sans_mono,
	// 	&mut font.ttf_font,
	// 	pos,
	// 	&mut commands
	// );

	// pos.x				+= 10.0;
	// spawn::file_text	(
	// 	"playground/salva_dfsph_solver.rs",
	// 	&font_handles.droid_sans_mono,
	// 	&mut font.ttf_font,
	// 	pos,
	// 	&mut commands
	// );

	commands.insert_resource(NextState(AppMode::Main));
}

pub fn setup_lighting_system(
	mut commands				: Commands,
) {
	const HALF_SIZE: f32		= 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 8000.0,
			// Configure the projection to better fit the scene
			shadow_projection	: OrthographicProjection {
				left			: -HALF_SIZE,
				right			:  HALF_SIZE,
				bottom			: -HALF_SIZE,
				top				:  HALF_SIZE,
				near			: -10.0 * HALF_SIZE,
				far				: 100.0 * HALF_SIZE,
				..default()
			},
			shadows_enabled		: true,
			..default()
		},
		transform				: Transform {
			translation			: Vec3::new(10.0, 2.0, 10.0),
			rotation			: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..default()
		},
		..default()
	});

	// commands
	//     .spawn_bundle(DirectionalLightBundle {
	//         ..Default::default()
	//     })
	//     .insert(Sun); // Marks the light as Sun

	//
}

pub fn setup_camera_system(
	mut query			: Query<&mut FlyCamera>
) {
}

pub fn setup_cursor_visibility_system(
	mut windows	: ResMut<Windows>,
	mut picking	: ResMut<PickingPluginsState>,
) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode	(true);
	window.set_cursor_visibility(false);

	picking.enable_picking 		= false;
	picking.enable_highlighting = false;
	picking.enable_interacting 	= false;
}

pub fn cursor_visibility_system(
	mut windows		: ResMut<Windows>,
	btn				: Res<Input<MouseButton>>,
	key				: Res<Input<KeyCode>>,
	time			: Res<Time>,
	mut q_camera	: Query<&mut FlyCamera>,
		app_mode	: Res<CurrentState<AppMode>>,
	mut picking		: ResMut<PickingPluginsState>,
	mut	commands	: Commands
) {
	let window 		= windows.get_primary_mut();
	if window.is_none() {
		return;
	}
	let window		= window.unwrap();
	let cursor_visible = window.cursor_visible();
	let window_id	= window.id();

	let mut set_cursor_visibility = |v| {
		window.set_cursor_visibility(v);
		window.set_cursor_lock_mode(!v);
	};

	let mut set_visibility = |v| {
		set_cursor_visibility(v);

		picking.enable_picking = v;
		picking.enable_highlighting = v;
		picking.enable_interacting = v;

		commands.insert_resource(NextState(
			if v { AppMode::Editor } else { AppMode::Main }
		));
	};

	if key.just_pressed(KeyCode::Escape) {
		let toggle 	= !cursor_visible;
		set_visibility(toggle);
	}

	if btn.just_pressed(MouseButton::Left) && app_mode.0 == AppMode::Main{
		set_cursor_visibility(false);
	}

	// #[cfg(debug_assertions)]
	if time.seconds_since_startup() > 1.0 {
		let is_editor = app_mode.0 == AppMode::Editor;
		set_cursor_visibility(is_editor);

		let mut camera 	= q_camera.single_mut();
		camera.enabled_rotation = !is_editor;
	}
}

pub fn input_system(
		btn			: Res<Input<MouseButton>>,
		key			: Res<Input<KeyCode>>,
		time		: Res<Time>,
	mut camera_ids	: ResMut<CameraIDs>,
	mut shadertoy_canvas : ResMut<ShadertoyCanvas>,
	mut exit		: EventWriter<AppExit>,
	mut q_camera	: Query<&mut Camera>,
	mut q_camera3d	: Query<&mut Camera3d>,
	mut q_fly_camera: Query<&mut FlyCamera>,
		q_selection	: Query<&Selection>,
		q_children	: Query<&Children>,
) {
	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key1) {
		// parse_source_file();
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key0) {
		let mut camera = q_camera.get_mut(camera_ids.camera2d.unwrap()).unwrap();
		let toggle = !camera.is_active;

		camera.is_active = toggle;

		let mut camera = q_camera.get_mut(camera_ids.camera3d.unwrap()).unwrap();
		camera.priority = if toggle { 1 } else { 0 };

		let mut camera3d = q_camera3d.get_mut(camera_ids.camera3d.unwrap()).unwrap();
		camera3d.clear_color = if toggle { ClearColorConfig::None } else { ClearColorConfig::Default };

		shadertoy_canvas.active = toggle;
	}

	if !q_fly_camera.is_empty() {
		let mut camera = q_fly_camera.single_mut();

		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
			let toggle 	= !camera.enabled_follow;
			camera.enabled_follow = toggle;
		}

		if key.just_pressed(KeyCode::Escape) {
			let toggle 	= !camera.enabled_rotation;
			camera.enabled_rotation = toggle;
		}
	}
}

fn check_selection_recursive(
	children	: &Children,
	q_children	: &Query<&Children>,
	q_selection : &Query<&Selection>,
	depth		: u32,
	max_depth 	: u32
 ) -> bool {
	let mut selection_found = false;
	for child in children.iter() {
		let selection = match q_selection.get(*child) {
			Ok(s) => s,
			Err(_) => continue,
		};

		if selection.selected() {
			selection_found = true;
		} else {
			if depth >= max_depth {
				continue;
			}
			let subchildren = q_children.get(*child);
			if subchildren.is_ok() {
				selection_found = check_selection_recursive(subchildren.unwrap(), q_children, q_selection, depth + 1, max_depth);
			}
		}

		if selection_found {
			break;
		}
	}

	selection_found
}

pub fn load_assets(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut ass				: ResMut<AssetServer>,
) {
	font_handles.droid_sans_mono = ass.load("fonts/droidsans-mono.ttf");
}

pub fn asset_loading_events(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut ev_asset		: EventReader<AssetEvent<TextMeshFont>>,
	mut commands		: Commands
) {
	for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
				// we only have 1 asset now so it's that simple
                if font_handles.droid_sans_mono == *handle {
					commands.insert_resource(NextState(AppMode::AssetsLoaded));
				}
            }
            AssetEvent::Modified { handle } => {
            }
            AssetEvent::Removed { handle } => {
            }
        }
    }
}

pub fn setup_shadertoy(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut st_res: ResMut<ShadertoyResources>,
) {
	let shadertoy_name = "nightsky";
	st_res.include_debugger = false;

	let all_shader_handles: ShaderHandles =
		bevy_shadertoy_wgsl::make_and_load_shaders3(shadertoy_name, &asset_server, st_res.include_debugger);

	commands.insert_resource(all_shader_handles);
}

pub fn despawn_system(mut commands: Commands, time: Res<Time>, mut despawn: ResMut<DespawnResource>) {
	if time.seconds_since_startup() > 0.1 {
		for entity in &despawn.entities {
//			println!("Despawning entity {:?}", entity);
			commands.entity(*entity).despawn_recursive();
		}
		despawn.entities.clear();
	}
}
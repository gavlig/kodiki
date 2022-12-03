use bevy				:: { prelude :: * };
use bevy				:: { app::AppExit };
use bevy				:: core_pipeline :: clear_color :: ClearColorConfig;
use bevy				:: window :: CursorGrabMode;
use bevy_reader_camera	:: { * };
use bevy_mod_picking	:: { * };
use iyes_loopless		:: { prelude :: * };
use bevy_shadertoy_wgsl	:: { * };

use bevy_debug_text_overlay :: { screen_print };
use bevy_polyline	:: prelude :: { * };

use crate :: bevy_ab_glyph :: ABGlyphFont;
use crate :: bevy_ab_glyph :: mesh_generator :: generate_glyph_mesh_dbg;

use super :: spawn :: WorldAxisDesc;
use super :: { * };

pub fn setup_world_system(
	mut camera_ids		: ResMut<CameraIDs>,
	mut	mesh_assets		: ResMut<Assets<Mesh>>,
	mut	material_assets : ResMut<Assets<StandardMaterial>>,

	mut commands		: Commands,
) {
	// spawn::infinite_grid(&mut commands);

	spawn::world_axis	(Transform::default(), WorldAxisDesc::default(), &mut mesh_assets, &mut material_assets, &mut commands);

	spawn::fixed_sphere	(Transform::default(), 0.02, Color::SEA_GREEN, &mut mesh_assets, &mut material_assets, &mut commands);

	spawn::camera(
		None,
		&mut camera_ids,
		&mut commands
	);

	commands.insert_resource(NextState(AppMode::Main));
}

pub fn setup_lighting_system(
	mut commands				: Commands,
) {
	const HALF_SIZE: f32		= 100.0;

	commands.spawn(DirectionalLightBundle {
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

fn set_cursor_visibility(v: bool, window: &mut Window) {
	window.set_cursor_visibility(v);
	window.set_cursor_grab_mode	(if v { CursorGrabMode::None } else { CursorGrabMode::Locked });
}

fn toggle_picking_mode(v: bool, mut picking: &mut PickingPluginsState) {
	picking.enable_picking = v;
	picking.enable_highlighting = v;
	picking.enable_interacting = v;
}

pub fn input_system(
		btn			: Res<Input<MouseButton>>,
		key			: Res<Input<KeyCode>>,
		mouse_state	: Res<MouseCursorState>,
		time		: Res<Time>,
	mut camera_ids	: ResMut<CameraIDs>,
	mut shadertoy_canvas : ResMut<ShadertoyCanvas>,
	mut exit		: EventWriter<AppExit>,
	mut q_camera	: Query<&mut Camera>,
	mut q_camera3d	: Query<&mut Camera3d>,
	mut q_reader_camera : Query<&mut ReaderCamera>,

	mut windows		: ResMut<Windows>,
	mut	commands	: Commands
) {
	let delta_seconds = time.delta_seconds();

	let window 		= windows.get_primary_mut();
	if window.is_none() {
		return;
	}
	let window		= window.unwrap();

	if key.pressed(KeyCode::LControl) && key.pressed(KeyCode::LAlt) && key.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key1) {
		// parse_source_file();
	}

	// turn off shadertoy on background
	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key0) {
		let mut camera = q_camera.get_mut(camera_ids.camera2d.unwrap()).unwrap();
		let toggle = !camera.is_active;

		// active 2d camera == visible shadertoy
		camera.is_active = toggle;

		let mut camera = q_camera.get_mut(camera_ids.camera3d.unwrap()).unwrap();
		// active 2d camera always has priority 0 so if it's disabled we need to give 3d camera priority 0
		camera.priority = if toggle { 1 } else { 0 };

		let mut camera3d = q_camera3d.get_mut(camera_ids.camera3d.unwrap()).unwrap();
		// without shadertoy we need to cleanup background
		camera3d.clear_color = if toggle { ClearColorConfig::None } else { ClearColorConfig::Default };

		// turn off compute shaders to increase fps as well
		shadertoy_canvas.active = toggle;
	}

	if !q_reader_camera.is_empty() {
		let mut camera = q_reader_camera.single_mut();

		// Reader mode
		if key.just_pressed(KeyCode::LAlt) && !key.pressed(KeyCode::LControl) {
			camera.set_restrictions(true, true, true);

			set_cursor_visibility(false, window);

			commands.insert_resource(NextState(AppMode::Reader));
		} else if key.just_released(KeyCode::LAlt) && camera.mode == CameraMode::Reader {
			camera.set_restrictions(false, false, false);

			set_cursor_visibility(true, window);

			commands.insert_resource(NextState(AppMode::Main));
		}

		// Fly mode
		if key.just_pressed(KeyCode::LControl) && key.pressed(KeyCode::LAlt) {
			camera.set_mode_wrestrictions(CameraMode::Fly, true, true, false);
			
			set_cursor_visibility(false, window);
			
			commands.insert_resource(NextState(AppMode::Fly));
		} else if key.just_released(KeyCode::LControl) && camera.mode == CameraMode::Fly {
			camera.set_mode_wrestrictions(CameraMode::Reader, false, false, true);
			
			set_cursor_visibility(true, window);
			
			commands.insert_resource(NextState(AppMode::Main));
		}

		if camera.mode == CameraMode::Reader {
			// if key.pressed(KeyCode::Left) {
			// 	camera.column_dec(delta_seconds);
			// }

			// if key.pressed(KeyCode::Right) {
			// 	camera.column_inc(delta_seconds);
			// }

			// if key.pressed(KeyCode::Up) {
			// 	camera.row_dec(delta_seconds);
			// }

			// if key.pressed(KeyCode::Down) {
			// 	camera.row_inc(delta_seconds);
			// }
		}
	}
}

pub fn stats_system(
	q_camera: Query<(&ReaderCamera, &Transform)>,
	// q_center_pick: Query<(&Transform, &Row, &Column), With<CenterPick>>
	q_text_descriptor: Query<&TextDescriptor>,
) {
	for (camera, transform) in q_camera.iter() {
		let (qw, qh) =
		if let Some(target) = camera.target {
			let descriptor = q_text_descriptor.get(target).unwrap();
			(descriptor.glyph_width, descriptor.glyph_height)
		} else {
			(0.0, 0.0)
		};

		screen_print!("row: {}({:.1}) col: {}({:.1}) zoom: {:.1} pitch: {:.1} glyph_w: {:.1} glyph_h: {:.1}",
			camera.row,
			camera.vertical_scroll,
			camera.column,
			camera.horizontal_scroll,
			camera.zoom,
			camera.pitch,
			qw,
			qh
		);

		screen_print!("camera transform. p: {:.2} {:.2} {:.2} q: {:.2} {:.2} {:.2} {:.2}",
			transform.translation.x,
			transform.translation.y,
			transform.translation.z,

			transform.rotation.x,
			transform.rotation.y,
			transform.rotation.z,
			transform.rotation.w
		);
	}

	// for (tform, row, column) in q_center_pick.iter() {
	// 	screen_print!("Center Pick: row: {} col: {} x: {}", row.0, column.0, tform.translation.x);
	// }
}

pub fn load_assets(
	mut font_handles	: ResMut<FontAssetHandles>,
		ass				: ResMut<AssetServer>,
) {
	font_handles.droid_sans_mono	= ass.load("fonts/droidsans-mono.ttf");
	font_handles.open_dyslexic		= ass.load("fonts/OpenDyslexic3-Regular.ttf");
	font_handles.source_code_pro	= ass.load("fonts/SourceCodePro-Regular.ttf");
	font_handles.B612				= ass.load("fonts/B612Mono-Regular.ttf");
	font_handles.share_tech			= ass.load("fonts/ShareTechMono-Regular.ttf");

	font_handles.ubuntu_mono		= ass.load("fonts/UbuntuMono-Regular.otf");
	font_handles.dejavu_serif		= ass.load("fonts/DejaVuSerif.ttf");

	font_handles.main				= font_handles.ubuntu_mono.clone_weak();
	font_handles.fallback			= font_handles.dejavu_serif.clone_weak();
}

pub fn asset_loading_events(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut ev_asset		: EventReader<AssetEvent<ABGlyphFont>>,
	mut commands		: Commands
) {
	for ev in ev_asset.iter() {
		match ev {
			AssetEvent::Created { handle } => {
				font_handles.loaded_cnt += 1;
				if font_handles.ubuntu_mono == *handle {
					println!("ubuntu_mono loaded!");
				}

				if font_handles.dejavu_serif == *handle {
					println!("dejavu serif loaded!");
				}
			}
			AssetEvent::Modified { handle: _ } => {
			}
			AssetEvent::Removed { handle: _ } => {
			}
		}
	}

	if font_handles.loaded_cnt == 7 {
		commands.insert_resource(NextState(AppMode::AssetsLoaded));
		println!("assets loaded!");
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
	if time.elapsed_seconds() > 0.1 {
		for entity in &despawn.entities {
			commands.entity(*entity).despawn_recursive();
		}
		despawn.entities.clear();
	}
}
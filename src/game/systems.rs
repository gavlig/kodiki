use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy			:: core_pipeline :: clear_color :: ClearColorConfig;
use bevy_fly_camera	:: { * };
use bevy_mod_picking:: { * };
use iyes_loopless	:: { prelude :: * };
use bevy_shadertoy_wgsl :: { * };
use bevy_debug_text_overlay :: { screen_print };

use bevy::render::mesh::shape as render_shape;

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
	// spawn::infinite_grid(&mut commands);

	spawn::world_axis	(&mut meshes, &mut materials, &mut commands);

	spawn::fixed_sphere	(Transform::identity(), 0.02, Color::SEA_GREEN, &mut meshes, &mut materials, &mut commands);

	let font_handle = &font_handles.share_tech;

	// without font we can't go further
	let font		= fonts.get_mut(font_handle).unwrap();
	
	let mut pos		= Vec3::new(0.0, 0.0, 0.0);
	let file_entity =
	text::spawn::file(
		"playground/herringbone_spawn.rs", // rustc_ast.rs",
		font_handle,
		&mut font.ttf_font,
		pos,
		&mut meshes,
		&mut materials,
		&mut commands
	);

	spawn::camera	(file_entity, &mut camera_ids, &mut commands);

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

	if !q_fly_camera.is_empty() {
		let mut camera = q_fly_camera.single_mut();

		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
			let toggle 	= !camera.enabled_reader;
			camera.enabled_reader = toggle;
		}

		if key.just_pressed(KeyCode::Escape) {
			let toggle 	= !camera.enabled_rotation;
			camera.enabled_rotation = toggle;
			camera.enabled_translation = toggle;
			camera.enabled_zoom = toggle;
		}

		if key.pressed(KeyCode::Left) {
			if camera.column > 0 {
				camera.column -= 1;
			}
		}

		if key.pressed(KeyCode::Right) {
			camera.column += 1;
		}

		if key.pressed(KeyCode::Up) {
			if camera.row > 0 {
				camera.row -= 1;
			}
		}

		if key.pressed(KeyCode::Down) {
			camera.row += 1;
		}
	}
}

pub fn stats_system(
	q_camera: Query<&FlyCamera>,
	// q_center_pick: Query<(&Transform, &Row, &Column), With<CenterPick>>
	q_reader_data: Query<&ReaderData>,
) {
	for fly_camera in q_camera.iter() {
		let (qw, qh) =
		if let Some(target) = fly_camera.target {
			let reader_data = q_reader_data.get(target).unwrap();
			(reader_data.glyph_width, reader_data.glyph_height)
		} else {
			(0.0, 0.0)
		};

		screen_print!("row: {}({:.3}) col: {}({:.3}) pitch: {:.3} glyph_w: {:.3} glyph_h: {:.3}",
			fly_camera.row,
			fly_camera.row_scroll_accum,
			fly_camera.column,
			fly_camera.column_scroll_accum,
			fly_camera.pitch,
			qw,
			qh
		);
	}

	// for (tform, row, column) in q_center_pick.iter() {
	// 	screen_print!("Center Pick: row: {} col: {} x: {}", row.0, column.0, tform.translation.x);
	// }
}

pub fn load_assets(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut ass				: ResMut<AssetServer>,
) {
	font_handles.droid_sans_mono = ass.load("fonts/droidsans-mono.ttf");
	font_handles.open_dyslexic = ass.load("fonts/OpenDyslexic3-Regular.ttf");
	font_handles.source_code_pro = ass.load("fonts/SourceCodePro-Regular.ttf");
	font_handles.B612 = ass.load("fonts/B612Mono-Regular.ttf");
	font_handles.share_tech = ass.load("fonts/ShareTechMono-Regular.ttf");
}

pub fn asset_loading_events(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut ev_asset		: EventReader<AssetEvent<TextMeshFont>>,
	mut commands		: Commands
) {
	for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
				font_handles.loaded_cnt += 1;
				if font_handles.droid_sans_mono == *handle {
					println!("droid sans loaded!");
				}

				if font_handles.open_dyslexic == *handle {
					println!("open_dyslexic loaded!");
				}

				if font_handles.source_code_pro == *handle {
					println!("source_code_pro loaded!");
				}
            }
            AssetEvent::Modified { handle: _ } => {
            }
            AssetEvent::Removed { handle: _ } => {
            }
        }
    }

	if font_handles.loaded_cnt == 5 {
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
	if time.seconds_since_startup() > 0.1 {
		for entity in &despawn.entities {
//			println!("Despawning entity {:?}", entity);
			commands.entity(*entity).despawn_recursive();
		}
		despawn.entities.clear();
	}
}
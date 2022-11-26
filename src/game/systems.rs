use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy			:: core_pipeline :: clear_color :: ClearColorConfig;
use bevy_fly_camera	:: { * };
use bevy_mod_picking:: { * };
use iyes_loopless	:: { prelude :: * };
use bevy_shadertoy_wgsl :: { * };

use bevy_debug_text_overlay :: { screen_print };
use bevy_polyline	:: prelude :: { * };

use crate			:: bevy_ab_glyph :: ABGlyphFont;

use super			:: spawn :: WorldAxisDesc;
use super           :: { * };
use crate			:: { bevy_helix };
use crate			:: { bevy_helix :: SurfacesMapBevy };
use crate			:: { bevy_helix :: SurfaceBevy };
use crate			:: { bevy_helix :: editor :: EditorViewBevy };
use crate           :: { bevy_ab_glyph :: TextMeshesCache };

use helix_term	:: compositor	:: SurfacesMap	as SurfacesMapHelix;

pub fn setup_world_system(
		surfaces_helix	: ResMut<SurfacesMapHelix>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
    mut mesh_cache  : ResMut<TextMeshesCache>,
		font_handles: Res<FontAssetHandles>,
	mut fonts		: ResMut<Assets<ABGlyphFont>>,
	mut camera_ids	: ResMut<CameraIDs>,

	mut	mesh_assets	: ResMut<Assets<Mesh>>,
	mut	material_assets : ResMut<Assets<StandardMaterial>>,

	mut commands	: Commands,
) {
	// spawn::infinite_grid(&mut commands);

	spawn::world_axis	(Transform::identity(), WorldAxisDesc::default(), &mut mesh_assets, &mut material_assets, &mut commands);

	spawn::fixed_sphere	(Transform::identity(), 0.02, Color::SEA_GREEN, &mut mesh_assets, &mut material_assets, &mut commands);

	// without font we can't go further
	let font_handle = &font_handles.ubuntu_mono;
	let font		= fonts.get_mut(font_handle).unwrap();
	
	let mut pos		= Vec3::new(0.0, 0.0, 0.0);

	for (layer_name, surface_helix) in surfaces_helix.iter() {
		if surfaces_bevy.contains_key(layer_name) {
			println!("setup_world_system: not creating surface {} because it already exists!", layer_name);
			continue;
		}

		let mut surface_bevy = SurfaceBevy::default();

		let layer_entity =
		bevy_helix::spawn::surface(
			layer_name,
			pos,

			&surface_helix,
			&mut surface_bevy,

			&font,

			&mut mesh_cache,

			mesh_assets.as_mut(),
			&mut commands
		);

		surface_bevy.entity = Some(layer_entity);
		surfaces_bevy.insert(layer_name.clone(), surface_bevy);
	}

	let surface_bevy_editor = surfaces_bevy.get(&String::from(EditorViewBevy::ID)).unwrap();

	spawn::camera(
		None,//surface_bevy_editor.entity,
		&mut camera_ids,
		&mut commands
	);

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
	mut mouse_state	: ResMut<MouseCursorState>,
	mut	commands	: Commands
) {
	let window 		= windows.get_primary_mut();
	if window.is_none() {
		return;
	}
	let window		= window.unwrap();
	mouse_state.visible = window.cursor_visible();
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

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
		let toggle 	= !mouse_state.visible;
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
		mouse_state	: Res<MouseCursorState>,
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
	let delta_seconds = time.delta_seconds();

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

	if !q_fly_camera.is_empty() {
		let mut camera = q_fly_camera.single_mut();

		// if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
		// 	let toggle 	= !camera.enabled_reader;
		// 	camera.enabled_reader = toggle;
		// }

		//camera.enabled_reader = !key.pressed(KeyCode::LAlt);

		// camera.enabled_rotation = true;
		camera.enabled_zoom = true;

		if camera.enabled_reader {
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
		} else {
			// camera.enabled_rotation = true;
			// camera.enabled_translation = true;
			// camera.enabled_zoom = true;
		}

		if key.just_released(KeyCode::Escape) {
			camera.enabled_translation = !mouse_state.visible; // 
			camera.enabled_rotation = !mouse_state.visible;
		}
	}
}

pub fn stats_system(
	q_camera: Query<&FlyCamera>,
	// q_center_pick: Query<(&Transform, &Row, &Column), With<CenterPick>>
	q_text_descriptor: Query<&TextDescriptor>,
) {
	for fly_camera in q_camera.iter() {
		let (qw, qh) =
		if let Some(target) = fly_camera.target {
			let descriptor = q_text_descriptor.get(target).unwrap();
			(descriptor.glyph_width, descriptor.glyph_height)
		} else {
			(0.0, 0.0)
		};

		screen_print!("row: {}({:.1}) col: {}({:.1}) zoom: {:.1} pitch: {:.1} glyph_w: {:.1} glyph_h: {:.1}",
			fly_camera.row,
			fly_camera.vertical_scroll,
			fly_camera.column,
			fly_camera.horizontal_scroll,
			fly_camera.zoom,
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

	font_handles.ubuntu_mono = ass.load("fonts/UbuntuMono-Regular.otf");
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
				if font_handles.droid_sans_mono == *handle {
					println!("droid sans loaded!");
				}

				if font_handles.open_dyslexic == *handle {
					println!("open_dyslexic loaded!");
				}

				if font_handles.source_code_pro == *handle {
					println!("source_code_pro loaded!");
				}

				if font_handles.ubuntu_mono == *handle {
					println!("ubuntu_mono loaded!");
				}
            }
            AssetEvent::Modified { handle: _ } => {
            }
            AssetEvent::Removed { handle: _ } => {
            }
        }
    }

	if font_handles.loaded_cnt == 6 {
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
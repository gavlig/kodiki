use bevy				:: { prelude :: * };
use bevy				:: { app::AppExit };
use bevy				:: core_pipeline :: clear_color :: ClearColorConfig;
use bevy				:: window :: { CursorGrabMode, WindowFocused };
use bevy_reader_camera	:: { * };
use bevy_mod_picking	:: { * };
use iyes_loopless		:: { prelude :: * };
use bevy_shadertoy_wgsl	:: { * };

use bevy_debug_text_overlay :: { screen_print };

use crate :: bevy_ab_glyph :: ABGlyphFont;
use crate :: bevy_ab_glyph :: glyph_generator :: generate_string_mesh;

use super :: spawn :: AxisDesc;
use super :: { * };

pub fn setup_world_system(
	mut camera_ids		: ResMut<CameraIDs>,
	mut	mesh_assets		: ResMut<Assets<Mesh>>,
	mut	material_assets : ResMut<Assets<StandardMaterial>>,

	mut commands		: Commands,
) {
	// spawn::infinite_grid(&mut commands);

	spawn::axis	(Transform::default(), AxisDesc::default(), &mut mesh_assets, &mut material_assets, &mut commands);

	spawn::fixed_sphere	(Transform::default(), 0.02, Color::SEA_GREEN, &mut mesh_assets, &mut material_assets, &mut commands);

	spawn::camera(
		None,
		&mut camera_ids,
		&mut commands
	);

	commands.insert_resource(NextState(AppMode::Main));
}

pub fn setup_ab_glyph_tests(
	font_assets			: Res<Assets<ABGlyphFont>>,
	font_handles		: Res<FontAssetHandles>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets : ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands,
) {
	let fonts = ABFonts::new(&font_assets, &font_handles);
	
	let text = String::from("test_text_abcd(){}:/@#$");
	let mut string_with_fonts = StringWithFonts::new();
	for c in text.chars() {
		let glyph_with_fonts = GlyphWithFonts::new(String::from(c), &fonts);
		string_with_fonts.push(glyph_with_fonts);
	}
	
	let mesh			= generate_string_mesh(&string_with_fonts, None);
	let mesh_handle		= mesh_assets.add(mesh);
	let material_handle	= material_assets.add(Color::WHITE.into());
	
	commands.spawn(PbrBundle {
		mesh		: mesh_handle,
		material	: material_handle,
		transform 	: Transform {
			translation	: Vec3::new(0.0, 0.0, 0.5),
			scale		: [fonts.main.scale; 3].into(),
			..default()
		},
		..default()
	});
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
	window.set_cursor_grab_mode	(if v { CursorGrabMode::None } else { CursorGrabMode::Confined });
}

fn toggle_picking_mode(v: bool, mut picking: &mut PickingPluginsState) {
	picking.enable_picking = v;
	picking.enable_highlighting = v;
	picking.enable_interacting = v;
}

pub fn input_system(
		key			: Res<Input<KeyCode>>,
		camera_ids	: Res<CameraIDs>,
	mut shadertoy_canvas : ResMut<ShadertoyCanvas>,
	mut exit		: EventWriter<AppExit>,
	mut q_camera	: Query<&mut Camera>,
	mut q_camera3d	: Query<&mut Camera3d>,
	mut q_reader_camera : Query<&mut ReaderCamera>,

	mut windows		: ResMut<Windows>,
	mut	commands	: Commands
) {
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
	}
}

// works in AppMode::Reader only
pub fn window_unfocused_system(
	mut	windows			: ResMut<Windows>,
    mut focused_events	: EventReader<WindowFocused>,
	mut q_reader_camera : Query<&mut ReaderCamera>,
	
	mut commands		: Commands,
) {
	let window = windows.get_primary_mut();
	let window = if window.is_none() { return; } else { window.unwrap() };
	
	if q_reader_camera.is_empty() {
		return;
	}
	
	let mut camera = q_reader_camera.single_mut();
	
    for e in focused_events.iter() {
		if e.id != window.id() || e.focused == true {
			continue;
		}
		
		if camera.mode == CameraMode::Reader {
			camera.set_restrictions(false, false, false);

			set_cursor_visibility(true, window);

			commands.insert_resource(NextState(AppMode::Main));
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

		screen_print!("visible rows: {:.2} row: {}({:.1}) offset: {} col: {}({:.1}) zoom: {:.1} pitch: {:.1} glyph_w: {:.1} glyph_h: {:.1}",
			camera.visible_rows,
			camera.row_offset_in + (camera.visible_rows / 2.0).floor() as u32,
			camera.vertical_scroll,
			camera.row_offset_in,
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
	let ubuntu_mono			= ass.load("fonts/UbuntuMono-Regular.otf");
	let noto_color_emoji	= ass.load("fonts/NotoColorEmoji.ttf");
	
	let mut fallback		= Vec::new();
	fallback.push			(ass.load("fonts/DejaVuSerif.ttf"));

	font_handles.main		= ubuntu_mono;
	font_handles.emoji		= noto_color_emoji;
	font_handles.fallback	= fallback;
}

pub fn font_asset_loading_events(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut ev_asset		: EventReader<AssetEvent<ABGlyphFont>>,
	mut commands		: Commands
) {
	for ev in ev_asset.iter() {
		match ev {
			AssetEvent::Created { handle: _ } => {
				font_handles.loaded_cnt += 1;
			}
			AssetEvent::Modified { handle: _ } => {
			}
			AssetEvent::Removed { handle: _ } => {
			}
		}
	}

	if font_handles.loaded_cnt == font_handles.fallback.len() + 2 {
		commands.insert_resource(NextState(AppMode::AssetsLoaded));
		println!("fonts loaded!");
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
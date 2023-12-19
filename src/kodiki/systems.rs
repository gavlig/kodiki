use bevy :: {
	prelude	:: *,
	gltf	:: Gltf,
	window	:: PrimaryWindow,
};

use bevy_reader_camera	:: *;
use bevy_rapier3d		:: prelude :: *;

#[cfg(feature = "stats")]
use bevy_debug_text_overlay :: screen_print;

use super :: { *, systems_util :: * };

#[cfg(feature = "debug")]
use super :: spawn :: AxisDesc;

use crate :: {
	z_order,
	bevy_wezterm	:: BevyWezTerm,
	bevy_helix		:: { HelixApp, TokioRuntime, utils :: * },
	bevy_ab_glyph	:: { ABGlyphFont, glyph_mesh_generator :: generate_string_mesh },
	kodiki_ui :: {
		text_cursor		:: CursorVisualAsset,
		text_surface	:: PathRowCol,
		raypick			:: Clicked,
		context_switcher:: { ContextSwitcher, ContextSwitcherEntry },
	},
};

pub fn setup_world(
	mut camera_ids		: ResMut<CameraIDs>,
	mut rapier_debug	: ResMut<DebugRenderContext>,
	mut next_state		: ResMut<NextState<AppMode>>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets : ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands,
) {
	#[cfg(feature = "debug")] {
		spawn::axis	(Transform::default(), AxisDesc::default(), &mut mesh_assets, &mut material_assets, &mut commands);
		spawn::fixed_sphere	(Transform::default(), 0.02, Color::SEA_GREEN, &mut mesh_assets, &mut material_assets, &mut commands);
	}

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	spawn::camera(
		None,
		&mut camera_ids,
		&mut commands
	);

	// Context Switcher between terminal and code/text editor

	let entry0 = ContextSwitcher::new_entry(
		"󱃖".into(),
		"Code Editor".into(),
		"Ctrl + 1".into(),
	);

	let entry1 = ContextSwitcher::new_entry(
		"".into(),
		"Terminal".into(),
		"Ctrl + 2".into(),
	);

	let spawned_entries = ContextSwitcher::spawn(
		0.2,
		[entry0, entry1].into(),
		&mut mesh_assets,
		&mut material_assets,
		&fonts,
		&mut commands
	);

	commands.entity(spawned_entries[0]).insert(AppContextSwitcher::Entry(AppContext::CodeEditor));
	commands.entity(spawned_entries[1]).insert(AppContextSwitcher::Entry(AppContext::Terminal));

	//

	rapier_debug.enabled = false;
	rapier_debug.pipeline.style.rigid_body_axes_length = 0.1;

	next_state.set(AppMode::Main);
}

pub fn apply_context_switcher_state(
	mut	q_app_context_switcher	: Query<(&AppContextSwitcher, &mut ContextSwitcherEntry)>,
	mut next_context			: ResMut<NextState<AppContext>>,
) {
	for (marker, mut switcher_entry) in q_app_context_switcher.iter_mut() {
		if !switcher_entry.is_triggered {
			continue;
		}

		match marker {
			AppContextSwitcher::Entry(AppContext::CodeEditor) => {
				next_context.set(AppContext::CodeEditor);
			},
			AppContextSwitcher::Entry(AppContext::Terminal) => {
				next_context.set(AppContext::Terminal);
			}
		}

		// trigger is processed now
		switcher_entry.is_triggered = false;
	}
}

pub fn highlight_active_context_switcher(
	mut	q_switcher_entry: Query<(&AppContextSwitcher, &mut ContextSwitcherEntry, &Transform, Entity)>,
		app_context		: Res<State<AppContext>>,
		next_context	: Res<NextState<AppContext>>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands,
) {
	if next_context.0.is_some() {
		return;
	}

	for (app_context_switcher, mut entry, transform, entity) in q_switcher_entry.iter_mut() {
		match app_context_switcher {
			AppContextSwitcher::Entry(entry_app_context) => {
				if entry_app_context == app_context.get() && !entry.is_active {
					entry.highlight(
						entity,
						transform,
						&mut color_materials_cache,
						&mut material_assets,
						&mut commands
					);
					entry.is_active = true;
				} else if entry_app_context != app_context.get() && entry.is_active {
					entry.unhighlight(
						entity,
						transform,
						&mut color_materials_cache,
						&mut material_assets,
						&mut commands
					);
					entry.is_active = false;
				}
			},
		}
	}
}

pub fn setup_ab_glyph_tests(
	font_assets			: Res<Assets<ABGlyphFont>>,
	font_handles		: Res<FontAssetHandles>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets : ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let text = String::from("test_text_abcd(){}:/@#$");
	let first_char_string = String::from(text.chars().next().unwrap());
	let glyph_with_fonts = GlyphWithFonts::new(&first_char_string, &fonts);

	let mesh			= generate_string_mesh(&text, glyph_with_fonts.current_font(), None);
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

pub fn kodiki_ui_sync(
	mut	kodiki_ui : ResMut<KodikiUI>,
		helix_app_option: Option<NonSend<HelixApp>>,
) {
	let Some(helix_app) = helix_app_option else { return };

	kodiki_ui.dark_theme = helix_app.dark_theme();

	let style = helix_app.editor.theme.get("ui.statusline");

	kodiki_ui.context_switch_color = color_from_helix(style.bg.unwrap_or_else(|| { helix_view::graphics::Color::Cyan }));
}

pub fn keyboard_input(
		key			: Res<Input<KeyCode>>,
	mut rapier_debug: ResMut<DebugRenderContext>,

	mut q_reader_camera 	: Query<&mut ReaderCamera>,
	mut q_window_primary	: Query<&mut Window, With<PrimaryWindow>>,

	mut next_camera_mode	: ResMut<NextState<AppCameraMode>>,
	mut next_context		: ResMut<NextState<AppContext>>,
) {
	if key.pressed(KeyCode::ControlLeft) && key.just_pressed(KeyCode::Key9) {
		rapier_debug.enabled = !rapier_debug.enabled;
	}

	if let Ok(mut window) = q_window_primary.get_single_mut() {
		if let Ok(mut camera) = q_reader_camera.get_single_mut() {
			handle_camera_mode(&mut camera, &mut window, &key, &mut next_camera_mode);
		}
	}

	let ctrl_pressed = key.pressed(KeyCode::ControlLeft) || key.pressed(KeyCode::ControlRight);

	if ctrl_pressed {
		// switch to Helix
		if key.just_pressed(KeyCode::Key1) {
			next_context.set(AppContext::CodeEditor);
		}

		// switch to WezTerm
		if key.just_pressed(KeyCode::Key2) {
			next_context.set(AppContext::Terminal);
		}
	}
}

fn handle_camera_mode(
	camera		: &mut ReaderCamera,
	window		: &mut Window,
	key			: &Input<KeyCode>,
	next_state	: &mut NextState<AppCameraMode>,
) {
	// let restrictions_are_default = camera.restrictions_are_default();
	let mode_is_reader = camera.mode == CameraMode::Reader;

	// Return to default mode
	// if key.just_released(KeyCode::LControl) {
	// 	if !restrictions_are_default {
	// 		camera.apply_default_restrictions();
	// 	}

	// 	if !mode_is_reader {
	// 		camera.mode = CameraMode::Reader;
	// 	}

	// 	set_cursor_visibility(true, window);

	// 	next_state.set(AppCameraMode::Main);

	// 	return;
	// }

    // Full Reader mode == allow scrolling with mouse movement + no cursor
    // if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::LAlt) {
	// 	camera.set_restrictions(true, true, true);

	// 	set_cursor_visibility(false, window);

	// 	commands.insert_resource(NextState(AppMode::Reader));
	// } else

	// if key.just_released(KeyCode::LAlt) && mode_is_reader {
	// 	camera.set_restrictions(false, false, false, true);

	// 	set_cursor_visibility(true, window);

	// 	next_state.set(AppCameraMode::Main);
	// }

	let keys_pressed = key.get_pressed().len();
	let single_key_pressed = keys_pressed == 1;

	let fly_mode = camera.mode == CameraMode::Fly;

    // Zoom-only Reader mode
    if key.just_pressed(KeyCode::ControlLeft) && single_key_pressed && !fly_mode {
		camera.set_restrictions(false, false, true, false);

		set_cursor_visibility(true, window);

		next_state.set(AppCameraMode::Reader);
	} else if key.just_released(KeyCode::ControlLeft) && mode_is_reader && !fly_mode {
		camera.apply_default_restrictions();

		set_cursor_visibility(true, window);

		next_state.set(AppCameraMode::Main);
	}

    // Fly mode

	let fly_mode_keys_pressed = key.pressed(KeyCode::ControlLeft) && key.just_pressed(KeyCode::Home) && keys_pressed == 2;
	let esc_pressed = key.pressed(KeyCode::Escape);

	if fly_mode_keys_pressed && camera.mode != CameraMode::Fly {
		camera.set_mode_wrestrictions(CameraMode::Fly, true, true, false, false);

		set_cursor_visibility(false, window);

		next_state.set(AppCameraMode::Fly);
	} else if (fly_mode_keys_pressed || esc_pressed) && camera.mode == CameraMode::Fly {
		camera.set_mode(CameraMode::Reader);
		camera.apply_default_restrictions();

		set_cursor_visibility(true, window);

		next_state.set(AppCameraMode::Main);
	}
}

// pub fn on_window_unfocused(
// 	mut focused_events	: EventReader<WindowFocused>,
// 	mut q_reader_camera : Query<&mut ReaderCamera>,
// 	mut q_window_primary : Query<(Entity, &mut Window), With<PrimaryWindow>>,

// 	mut	next_state : ResMut<NextState<AppCameraMode>>,
// ) {
// 	let (window_entity, mut window) = if let Ok((e, w)) = q_window_primary.get_single_mut() { (e, w) } else { return };

// 	let Ok(mut camera) = q_reader_camera.get_single_mut() else { return };

// 	for e in focused_events.iter() {
// 		if e.window != window_entity || e.focused == true {
// 			continue;
// 		}

// 		camera.mode = CameraMode::Reader;
// 		camera.set_restrictions(false, false, false, true);
// 		set_cursor_visibility(true, &mut window);

// 		next_state.set(AppCameraMode::Main);
// 	}
// }

pub fn update_window_title(
	mut q_window_primary	: Query<&mut Window, With<PrimaryWindow>>,
		q_terminal			: Query<&BevyWezTerm>,
		app_context			: Res<State<AppContext>>,
		helix_app_option	: Option<NonSend<HelixApp>>,
) {
	let Ok(mut window) = q_window_primary.get_single_mut() else { return };

	match app_context.get() {
		AppContext::CodeEditor => {
			let Some(helix_app) = helix_app_option else { return };
			if helix_app.should_render() {
				window.title = helix_app.window_title();
			}
		},
		AppContext::Terminal => {
			for terminal in q_terminal.iter() {
				if terminal.active() && terminal.state_changed() {
					window.title = terminal.window_title();
				}
			}
		}
	}
}

#[cfg(feature = "stats")]
pub fn stats(
	q_camera: Query<(&ReaderCamera, &Transform)>,
	// q_center_pick: Query<(&Transform, &Row, &Column), With<CenterPick>>
	q_text_descriptor: Query<&TextDescriptor>,
) {
	for (camera, transform) in q_camera.iter() {
		let (qw, qh) =
		if let Some(target) = camera.target_object {
			let descriptor = q_text_descriptor.get(target).unwrap();
			(descriptor.glyph_width, descriptor.glyph_height)
		} else {
			(0.0, 0.0)
		};

		screen_print!("visible rows: {:.2} row: {}({:.1}) offset: {} col: {}({:.1}) zoom: {:.1} pitch: {:.1} glyph_w: {:.1} glyph_h: {:.1}",
			camera.visible_rows,
			camera.row_offset_in + (camera.visible_rows / 2.0).floor() as u32,
			camera.scroll,
			camera.row_offset_in,
			camera.column,
			camera.swipe,
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

#[cfg(not(feature = "stats"))]
pub fn stats() {}

pub fn load_assets(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut cursor_asset	: ResMut<CursorVisualAsset>,
		ass				: ResMut<AssetServer>,
) {
	let ubuntu_mono			= ass.load("fonts/UbuntuMonoNerdFont-Regular.ttf");
	let noto_color_emoji	= ass.load("fonts/NotoColorEmoji.ttf");

	let mut fallback		= Vec::new();
	fallback.push			(ass.load("fonts/DejaVuSerif.ttf"));

	font_handles.main		= ubuntu_mono;
	font_handles.emoji		= noto_color_emoji;
	font_handles.fallback	= fallback;

	cursor_asset.handle		= ass.load("meshes/cursor/default.gltf");
}

pub fn font_asset_loading_events(
	mut font_handles	: ResMut<FontAssetHandles>,
	mut ev_asset		: EventReader<AssetEvent<ABGlyphFont>>,
) {
	let fonts_were_loaded = font_handles.loaded();

	for ev in ev_asset.read() {
		match ev {
			AssetEvent::LoadedWithDependencies { id } => {
				font_handles.loaded_cnt += 1;
				
				if id == &font_handles.main.id() {
					println!("main font loaded! (fonts/UbuntuMonoNerdFont-Regular.ttf)");
				} else if id == &font_handles.emoji.id() {
					println!("emoji font loaded! (fonts/NotoColorEmoji.ttf)");
				} else if let Some(fallback_handle) = font_handles.fallback.first() {
					if id == &fallback_handle.id() {
						println!("fallback font loaded! (fonts/DejaVuSerif.ttf)");
					}
				}
			}
			_ => {},
		}
	}

	if !fonts_were_loaded && font_handles.loaded() {
		println!("font assets loaded!");
	}
}

pub fn gltf_asset_loading_events(
	mut	cursor_asset	: ResMut<CursorVisualAsset>,
	mut ev_asset		: EventReader<AssetEvent<Gltf>>,
) {
	let cursor_was_loaded = cursor_asset.loaded;

	for ev in ev_asset.read() {
		match ev {
			AssetEvent::LoadedWithDependencies { id } => {
				if cursor_asset.handle.id() == *id {
					cursor_asset.loaded = true;
				}
			},
			_ => {}
		}
	}

	if !cursor_was_loaded && cursor_asset.loaded {
		println!("gltf assets loaded!");
	}
}

pub fn asset_loading_tracking(
		font_handles	: Res<FontAssetHandles>,
		cursor_asset	: Res<CursorVisualAsset>,
	mut next_state 		: ResMut<NextState<AppMode>>,
) {
	if font_handles.loaded() && cursor_asset.loaded {
		next_state.set(AppMode::AssetsLoaded);
		println!("all assets loaded!");
	}
}

pub fn spawn_first_terminal(
		q_terminal		: Query<&BevyWezTerm>,
	mut q_camera		: Query<(&mut ReaderCamera, &Transform)>,
	mut commands		: Commands,

	(mut gltf_assets, mut cursor_asset)		: (ResMut<Assets<Gltf>>, ResMut<CursorVisualAsset>),
	(mut mesh_assets, mut material_assets)	: (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
	(font_assets, font_handles)				: (Res<Assets<ABGlyphFont>>, Res<FontAssetHandles>),
) {
	if !q_terminal.is_empty() { return }

	// this is fragile but will work as long as code editor (helix in our case currently) uses std::env::set_current_dir/current_dir
    let Ok(cwd) = std::env::current_dir() else { return };

	let font = font_assets.get(&font_handles.main).unwrap();

	let rows = 24;
	let cols = 150;

	let Ok((mut reader_camera, camera_transform)) = q_camera.get_single_mut() else { return };

	let column_width = font.horizontal_advance_mono();

	// putting terminal where the camera is currently 
	let x = camera_transform.translation.x + (-column_width * (cols as f32 / 2.0));
	let y = camera_transform.translation.y + reader_camera.y_top; // NOTE: surface anchor is not accounted for
	let z = z_order::surface::base();

	let translation = Vec3::new(x, y, z);

	let terminal_entity = BevyWezTerm::spawn(
		"Bevy Terminal",
		Some(cwd),
		font,
		rows,
		cols,
		Some(translation),
		&mut gltf_assets,
		&mut cursor_asset,
		&mut mesh_assets,
		&mut material_assets,
		&mut commands
	);

	reader_camera.target_entity = Some(terminal_entity);
}

pub fn process_clicked_terminal_path(
		q_clicked		: Query<(Entity, &PathRowCol), With<Clicked>>,
	mut next_context	: ResMut<NextState<AppContext>>,
		tokio_runtime	: Res<TokioRuntime>,
		helix_app_option: Option<NonSendMut<HelixApp>>,
	mut commands		: Commands,
) {
	let Some(mut helix_app) = helix_app_option else { return };
	
	let Ok((clicked_entity, path)) = q_clicked.get_single() else { return };
	
	next_context.set(AppContext::CodeEditor);

	let row = path.row.saturating_sub(1); // helix indexing
	let col = path.col.saturating_sub(1);
	
	tokio_runtime.block_on(
		helix_app.jump_to_path(&path.file_path, Some(row), Some(col))
	);
	
	commands.entity(clicked_entity).remove::<Clicked>();
}

pub fn despawn(mut commands: Commands, time: Res<Time>, mut despawn: ResMut<DespawnResource>) {
	if time.elapsed_seconds() > 0.1 {
		for entity in &despawn.recursive {
			commands.entity(*entity).despawn_recursive();
		}
		despawn.recursive.clear();

		for entity in &despawn.children_only {
			commands.entity(*entity).despawn_descendants();
			commands.entity(*entity).remove::<Children>();
		}
		despawn.children_only.clear();
	}
}
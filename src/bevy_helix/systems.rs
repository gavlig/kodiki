use bevy :: {
	prelude :: *,
	app		:: AppExit,
	gltf	:: Gltf,
	input	:: { keyboard :: * , mouse :: * },
	window	:: { WindowCloseRequested, PrimaryWindow },
};
use bevy_tweening		:: *;
use bevy_reader_camera	:: ReaderCamera;
// use bevy_vfx_bag		:: post_processing :: masks :: { Mask, MaskVariant };

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines	:: *;

#[cfg(feature = "tracing")]
use bevy_puffin :: *;

use helix_term :: {
	config	:: { Config, ConfigLoadError },
	args	:: Args,
	ui		:: EditorView,
};

use helix_view :: graphics :: { Rect, Color as HelixColor };

use helix_tui :: buffer :: { Buffer as SurfaceHelix, SurfaceFlags, SurfaceAnchor };

use anyhow :: { Context, Error, Result };

use std :: time :: { Instant, Duration };

use super :: {
	input,
	helix_app :: HelixApp,

	TokioRuntime,
	BevyHelixSettings,
	GotoDefinitionHighlight,

	MatchesMapCache, SearchKind,
	ArrowKeysState, KeyPressTiming, MousePosState, MouseButtonState, MouseHoverState,

	systems_util	:: *,
	surface			:: *,
	minimap			:: *,
	utils			:: *,
};

use crate :: {
	z_order,
	kodiki :: DespawnResource,
	kodiki_ui :: {
		*,
		text_cursor	:: *,
		color		:: *,
		tween_lens	:: *,
		raypick		:: *,
		resizer		:: *,
	},
	bevy_framerate_manager :: { FramerateManager, FramerateMode },
	bevy_ab_glyph :: { ABGlyphFont, FontAssetHandles, ABGlyphFonts },
};


pub fn startup_app(
	world: &mut World,
) {
	let mut surfaces_helix	= SurfacesMapHelix::default();
	let 	surfaces_bevy	= SurfacesMapBevy::default();

	let rect = Rect {
		x : 0,
		y : 0,
		width : 130,
		height : 60,
	};

	let surface_editor = SurfaceHelix::empty_with_spatial(rect, SurfaceFlags::default());
	surfaces_helix.insert(String::from(EditorView::ID),	surface_editor);

	world.insert_resource(surfaces_helix);
	world.insert_resource(surfaces_bevy);

	let tokio_runtime : &TokioRuntime = world.resource();

	let app = tokio_runtime.block_on(startup_impl(rect));

	world.insert_non_send_resource(app.unwrap());
}

async fn startup_impl(area: Rect) -> Result<HelixApp, Error> {
	let args = Args::parse_args().context("could not parse arguments").unwrap();

	let config_dir = helix_loader::config_dir();
	if !config_dir.exists() {
		std::fs::create_dir_all(&config_dir).ok();
	}

	helix_loader::initialize_config_file(args.config_file.clone());

	let config = match Config::load_default() {
		Ok(config) => config,
		Err(ConfigLoadError::Error(err)) if err.kind() == std::io::ErrorKind::NotFound => {
			Config::default()
		}
		Err(ConfigLoadError::Error(err)) => {
			eprintln!("Config Load Error: {}", err);
			Config::default()
		},
		Err(ConfigLoadError::BadConfig(err)) => {
			eprintln!("Bad config: {}", err);
			Config::default()
		}
	};

	let syn_loader_conf = helix_core::config::user_syntax_loader().unwrap_or_else(|err| {
		eprintln!("Bad language config: {}", err);
		helix_core::config::default_syntax_loader()
	});

	HelixApp::new(args, config, syn_loader_conf, area).context("unable to create new application")
}

pub fn startup_spawn(
		surfaces_helix	: Res<SurfacesMapHelix>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
	mut q_reader_camera	: Query<&mut ReaderCamera>,

	(mut mesh_assets, mut material_assets) : (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),

	mut commands		: Commands,
		app_option		: Option<NonSendMut<HelixApp>>,
) {
	let Some(app) = app_option else { panic!("HelixApp resource is not available in startup_spawn!") };

	let surface_editor_name = String::from(EditorView::ID);

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let minimap_entity = Minimap::spawn(
		&mut mesh_assets,
		&mut material_assets,
		&mut commands
	);

	let area = UVec2::new(app.editor_area().width as u32, app.editor_area().height as u32);
	
	let resizer_entity = Resizer::spawn(
		surface_editor_name.as_str(),
		area,
		&mut mesh_assets,
		&mut material_assets,
		&mut commands
	);
	
	let surface_helix_editor = surfaces_helix.get(&surface_editor_name).unwrap();

	let surface_bevy_editor = SurfaceBevy::spawn(
		&surface_editor_name,
		None,
		true, /* editor */
		true, /* scroll_enabled */
		Some(resizer_entity),
		&surface_helix_editor,
		fonts.main,
		&mut mesh_assets,
		&mut commands
	);

	commands.entity(surface_bevy_editor.entity).push_children(&[minimap_entity, resizer_entity]);

	let mut camera		= q_reader_camera.single_mut();

	camera.row_constant_offset = -1.0; // a cheat for top panel which height is 1 row

	camera.target_entity = Some(surface_bevy_editor.entity);
	camera.column		= (surface_bevy_editor.area.width / 2) as usize;

	surfaces_bevy.insert(surface_editor_name.clone(), surface_bevy_editor);
}

pub fn camera_update(
	mut q_camera		: Query<&mut ReaderCamera>,
		app_option		: Option<NonSendMut<HelixApp>>,
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let mut reader_camera = q_camera.single_mut();

	// scrolling offset coming from camera to Helix
	if reader_camera.row_offset_delta() != 0 {
		app.scroll(reader_camera.row_offset_delta_apply());
	}

	// make camera aware of row offset in Helix
	reader_camera.set_row_offset_in(app.row_offset_internal() as u32);

	// make Helix aware of row offset in camera to render viewport accordingly
	app.set_row_offset_external(reader_camera.row_offset_out() as usize);
}

pub fn render_helix(
		q_camera			: Query<&ReaderCamera>,
	mut surfaces_helix		: ResMut<SurfacesMapHelix>,
	mut framerate_manager	: ResMut<FramerateManager>,
		app_option			: Option<NonSendMut<HelixApp>>,
) {
	let Some(mut app) = app_option else { return };

	if app.should_close() { return }

	if app.should_render() {
		framerate_manager.request_active_framerate("Helix requested render".into());
	}

	let Ok(reader_camera) = q_camera.get_single() else { return };

	// surface area depends on camera frustum
	app.resize_editor_height(reader_camera.visible_rows.ceil() as u16 + 1); // + 1 to make sure we don't show empty rows on camera when scrolling
	app.resize_screen_width	(reader_camera.visible_columns.floor() as u16);
	app.render(&mut surfaces_helix);
}

#[cfg(feature = "stats")]
pub fn update_debug_stats(
	surfaces_helix		: Res<SurfacesMapHelix>,
	surfaces_bevy		: Res<SurfacesMapBevy>,
) {
	screen_print_active_layers	(&surfaces_helix);
	screen_print_stats			(&surfaces_bevy);
}

#[cfg(not(feature = "stats"))]
pub fn update_debug_stats() {}

pub fn manage_surfaces(
	mut surfaces_helix	: ResMut<SurfacesMapHelix>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,

	q_transform			: Query<&Transform>,
	q_camera			: Query<(Entity, &ReaderCamera)>,

	mut mesh_assets		: ResMut<Assets<Mesh>>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,

	mut despawn			: ResMut<DespawnResource>,
	mut commands		: Commands,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let (camera_entity, reader_camera) = q_camera.single();
	let camera_transform = q_transform.get(camera_entity).unwrap();

	despawn_unused_surfaces(
		&mut surfaces_helix,
		&mut surfaces_bevy,
		&mut despawn
	);

	spawn_new_surfaces(
		&mut surfaces_helix,
		&mut surfaces_bevy,
		reader_camera,
		camera_transform,
		fonts.main,
		&mut mesh_assets,
		&mut commands
	);
}

pub fn manage_cursors(
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,

	(mut gltf_assets, mut material_assets, mut cursor_asset)
						:
	(ResMut<Assets<Gltf>>, ResMut<Assets<StandardMaterial>>, ResMut<CursorVisualAsset>),

	mut despawn			: ResMut<DespawnResource>,
	mut commands		: Commands,
		app_option		: Option<NonSend<HelixApp>>,
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() || !app.editor_focused() { return }

	let (cursor_positions, cursor_surface_name) = if let Some(cursors) = app.cursor() { cursors } else { return };

	let surface_bevy = if let Some(surface) = surfaces_bevy.get_mut(&String::from(cursor_surface_name)) { surface } else { return };

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);
	let cursors_cnt = cursor_positions.len();

	// spawn new ones
	while surface_bevy.cursor_entities.len() < cursors_cnt {
		let cursor_entity = TextCursor::spawn(
			&surface_bevy.name,
			z_order::surface::cursor(),
			fonts.main,
			&mut gltf_assets,
			&mut material_assets,
			&mut cursor_asset,
			&mut commands
		);

		surface_bevy.cursor_entities.push(cursor_entity);

		commands.entity(surface_bevy.entity).add_child(cursor_entity);
	}

	// despawn old ones if Helix has less cursors than us, but keep the last one since helix returns cursor only if its visible
	while surface_bevy.cursor_entities.len() > cursors_cnt {
		despawn.recursive.push(surface_bevy.cursor_entities.pop().unwrap());

		if surface_bevy.cursor_entities.len() == 1 {
			break;
		}
	}
}

pub fn update_cursor(
		surfaces_bevy		: Res<SurfacesMapBevy>,
	mut	q_cursor			: Query<&mut TextCursor>,
		app_option			: Option<NonSend<HelixApp>>,
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() || !app.editor_focused() { return }

	let (cursor_positions, cursor_surface_name) = if let Some(cursors) = app.cursor() { cursors } else { return };

	// there can be no cursor visible in current viewport
	if cursor_positions.is_empty() { return }

	let surface_bevy = if let Some(surface) = surfaces_bevy.get(&String::from(cursor_surface_name)) { surface } else { return };

	let cursor_theme = app.editor.theme.get("ui.cursor");
	let cursor_color = if let Some(bg_color) = cursor_theme.bg {
		color_from_helix(bg_color)
	} else {
		Color::WHITE
	};

	let row_offset_dir = if surface_bevy.anchor.contains(SurfaceAnchor::Bottom) {
		RowOffsetDirection::Up
	} else {
		RowOffsetDirection::Down
	};
	let row_offset_sign = row_offset_dir.sign();

	for (cursor_index, cursor_entity) in surface_bevy.cursor_entities.iter().enumerate() {
		let mut cursor = if let Ok(cu) = q_cursor.get_mut(*cursor_entity) { cu } else { continue };

		cursor.color = cursor_color;
		cursor.row = cursor_positions[cursor_index].row;
		cursor.col = cursor_positions[cursor_index].col;
		cursor.row_offset_sign = row_offset_sign;

		cursor.blink_alpha = if app.dark_theme() { 0.2 } else { 0.8 };
	}
}

pub fn update_editor_resizer(
	mut q_resizer	: Query<(&mut Resizer, &mut Transform), Without<ReaderCamera>>,
		q_camera	: Query<&Transform, With<ReaderCamera>>,

	surfaces_bevy	: Res<SurfacesMapBevy>,
	app_option		: Option<NonSendMut<HelixApp>>,
) {
	let Some(mut app) = app_option else { return };

	if app.should_close() { return }

	let Some(editor_surface) = surfaces_bevy.get(&String::from(EditorView::ID)) else { return };
	let Some(resizer_entity) = editor_surface.resizer_entity else { return };

	let Ok((mut resizer, mut resizer_transform)) = q_resizer.get_mut(resizer_entity) else { return };

	// resizer -> helix : editor area is taken from resizer

	app.resize_editor_width(resizer.area.x as u16);

	// helix -> resizer : all colors are take from helix themes
	
	let style			= app.editor.theme.get("ui.statusline");

	resizer.quad_color	= color_from_helix(style.bg.unwrap_or_else(|| { HelixColor::Cyan }));

	let Ok(camera_transform) = q_camera.get_single() else { return };

	resizer_transform.translation.x = -resizer.width / 2.0 - resizer.margin;
	resizer_transform.translation.y = camera_transform.translation.y;
}

pub fn update_search_matches(
	mut matches_cache	: ResMut<MatchesMapCache>,
		key				: Res<ButtonInput<KeyCode>>,
		app_option		: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let new_search_pattern = app.active_search_pattern();

	let doc = app.current_document();
	let theme = app.editor.theme.name();

	let search_matches = matches_cache.map.get_mut(&SearchKind::Common).unwrap();

	// clean up last search pattern on esc
	if key.just_released(KeyCode::Escape) || search_matches.cache_outdated(doc, theme) {
		search_matches.clear();
	}

	if let Some(new_pattern) = new_search_pattern {
		if search_matches.update_required(&new_pattern, doc, theme) {
			let search_config = &app.editor.config().search;

			let pattern = new_pattern.as_str();

			search_matches.update(pattern, doc, theme, search_config, /* ignore_case= */true);
		}
	}
}

pub fn update_selection_search_matches(
	mut matches_cache	: ResMut<MatchesMapCache>,
		app_option		: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	// highlight currently selected text as if it was searched but only if it's on 1 line and from 1 cursor
	let (view, doc) = app.current_ref();
	let selection = doc.selection(view.id);

	let range = { // scoping to avoid having two references to matches_cache
		let selection_search_matches = matches_cache.map.get_mut(&SearchKind::Selection).unwrap();

		if selection.len() != 1 {
			selection_search_matches.clear();
			return
		}

		let range = selection.ranges()[0];

		let doc_slice = doc.text().slice(..);
		let line_range = range.line_range(doc_slice);

		// dont highlight cursor or multiline selection
		if range.len() < 2 || line_range.0 != line_range.1 {
			selection_search_matches.clear();
			return
		}

		let selection_slice = range.slice(doc_slice);

		let mut whitespace_only = true;
		for char in selection_slice.chars() {
			if !char.is_whitespace() {
				whitespace_only = false;
				break;
			}
		}

		if whitespace_only {
			selection_search_matches.clear();
			return
		}

		range
	};

	let (begin, end) = match range.direction() {
		helix_core::movement::Direction::Forward	=> (range.anchor, range.head),
		helix_core::movement::Direction::Backward	=> (range.head, range.anchor),
	};

	let theme = app.editor.theme.name();
	let new_selection_search_pattern = doc.text().slice(begin..end).to_string();

	let search_pattern_already_active = {
		let search_matches = matches_cache.map.get(&SearchKind::Common).unwrap();
		if let Some(cache) = search_matches.cache.as_ref() {
			cache.string.to_lowercase() == new_selection_search_pattern.to_lowercase()
		} else {
			false
		}
	};

	let selection_search_matches = matches_cache.map.get_mut(&SearchKind::Selection).unwrap();

	if search_pattern_already_active {
		selection_search_matches.clear();
	} else if selection_search_matches.update_required(&new_selection_search_pattern, doc, theme) {
		let pattern = new_selection_search_pattern.as_str();
		let search_config = &app.editor.config().search;

		selection_search_matches.update(pattern, doc, theme, search_config, /* ignore_case= */true);
	}
}

// pub fn helix_mode_effect(
// 	mut q_camera	: Query<Entity, With<ReaderCamera>>,
// 		q_mask		: Query<&Mask>,
// 		q_mask_animator_in	: Query<&Animator<Mask>, Without<MaskFadingOut>>,
// 		q_mask_animator_out	: Query<&Animator<Mask>, Without<MaskFadingIn>>,
// 		app_option	: Option<NonSend<HelixApp>>,
// 	mut commands	: Commands
// ) {
// 	let app = if let Some(app) = app_option { app } else { return };

// 	if app.should_close() { return }

// 	let helix_mode = app.mode();

// 	let camera_entity = q_camera.single_mut();

// 	let mask_animator_in = q_mask_animator_in.get(camera_entity);
// 	let mask_animator_out = q_mask_animator_out.get(camera_entity);
// 	let mask = q_mask.get(camera_entity);

// 	let make_tween = |
// 		start_strength	: f32,
// 		end_strength	: f32,
// 		start_fade		: f32,
// 		end_fade		: f32
// 	| -> Tween<Mask> {
// 		Tween::new(
// 			EaseFunction::ExponentialInOut,
// 			Duration::from_millis(300),
// 			MaskLens {
// 				start_strength,
// 				end_strength,
// 				start_fade,
// 				end_fade
// 			}
// 		)
// 	};

// 	let full_strength		= 1_000_000.;
// 	let zero_strength		= 1_000_000_000.; // almost no effect
// 	let fade_to_hidden		= 1.0;
// 	let fade_to_visible		= 0.0;

// 	// we're in insert mode and mask is not yet enabled
// 	if Mode::Insert == helix_mode && mask.is_err() {
// 		let start_strength	= zero_strength;
// 		let end_strength	= full_strength;
// 		let start_fade		= fade_to_hidden;
// 		let end_fade		= fade_to_visible;

// 		commands.entity(camera_entity).insert((
// 			Mask {
// 				strength	: start_strength,
// 				fade		: start_fade,
// 				variant		: MaskVariant::Crt,
// 			},
// 			Animator::new(make_tween(start_strength, end_strength, start_fade, end_fade)),
// 			MaskFadingIn
// 		));
// 	// we're in insert mode but there is an active fading out animation so we replace it with fading in
// 	} else if Mode::Insert == helix_mode && mask.is_ok() && mask_animator_in.is_err() && mask_animator_out.is_ok() {
// 		let mask			= mask.unwrap();
// 		let end_strength	= full_strength;
// 		let end_fade		= fade_to_visible;

// 		commands.entity(camera_entity)
// 			.insert((
// 				Animator::new(make_tween(mask.strength, end_strength, mask.fade, end_fade)),
// 				MaskFadingIn
// 			))
// 			.remove::<MaskFadingOut>();
// 	// we're in any other mode and mask is enabled
// 	} else if Mode::Insert != helix_mode && mask.is_ok() && mask_animator_out.is_err() {
// 		let mask			= mask.unwrap();
// 		let end_strength	= zero_strength;
// 		let end_fade		= 1.0;

// 		commands.entity(camera_entity)
// 			.insert((
// 				Animator::new(make_tween(mask.strength, end_strength, mask.fade, end_fade)),
// 				MaskFadingOut
// 			))
// 			.remove::<MaskFadingIn>();
// 	}
// }

// pub fn helix_mode_tween_events(
// 		q_mask_animator	: Query<(Entity, &Animator<Mask>)>,
// 		app_option	: Option<NonSend<HelixApp>>,
// 	mut commands	: Commands
// ) {
// 	let app = if let Some(app) = app_option { app } else { return };

// 	if app.should_close() { return }

// 	let helix_mode = app.mode();

// 	// remove Mask component from camera after animation is done
// 	for (entity, animator) in q_mask_animator.iter() {
// 		if animator.tweenable().progress() >= 1.0 {
// 			if helix_mode != Mode::Insert {
// 				commands.entity(entity).remove::<Mask>();
// 			}

// 			commands.entity(entity)
// 				.remove::<Animator<Mask>>()
// 				.remove::<MaskFadingIn>()
// 				.remove::<MaskFadingOut>()
// 			;
// 		}
// 	}
// }

pub fn mouse_last_clicked(
	mut mouse_button_state	: ResMut<MouseButtonState>,
		mouse_button		: Res<ButtonInput<MouseButton>>,
		framerate_manager	: Res<FramerateManager>,
) {
	// with idle fps we sometimes get a release event in the same frame when it was pressed. It's a dirty workaround, but will do for now
	if framerate_manager.mode() == FramerateMode::Idle {
		return;
	}

	for just_released in mouse_button.get_just_released() {
		mouse_button_state.last_clicked.insert(*just_released, Instant::now());
	}
}

pub fn mouse_hover(
	mut cursor_events	: EventReader<CursorMoved>,
	mut mouse_hover		: ResMut<MouseHoverState>,
		mouse_pos		: Res<MousePosState>,
		time			: Res<Time>,
		app_option		: Option<NonSendMut<HelixApp>>,
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let mouse_moved = !cursor_events.is_empty();
	cursor_events.clear();
	let cursor_coordinates_changed = mouse_pos.col != mouse_hover.col || mouse_pos.row != mouse_hover.row;

	if !app.editor_focused() || mouse_pos.surface_name != EditorView::ID {
		// TODO: we really need to do this only if hover widget is shown, need a flag somewhere or something
		if mouse_moved && cursor_coordinates_changed {
			app.hover_close();
		}
		return
	}

	// TODO: maybe bake node id into words when obtaining text from Helix?
	let get_hovered_node_id = |app: &HelixApp| -> Option<usize> {
		let (view, doc) = app.current_ref();
		let syntax = if let Some(syntax) = doc.syntax() { syntax } else { return None };

		let char_idx =
		if let Some(char_idx) = view.pos_at_screen_coords(
			doc,
			mouse_pos.row,
			mouse_pos.col,
			true
		) { char_idx } else { return None };

		let text = doc.text().slice(..);
		let byte = text.char_to_byte(char_idx);

		if let Some(node) = syntax.tree().root_node().descendant_for_byte_range(byte, byte) {
			if match node.kind() {
				"identifier"		=> true,
				"type_identifier"	=> true,
				"field_identifier"	=> true,
				_					=> false,
			} {	Some(node.id())	} else { None }
		} else {
			None
		}
	};

	if !mouse_moved {
		if mouse_hover.timer.tick(time.delta()).just_finished() {
			mouse_hover.row = mouse_pos.row;
			mouse_hover.col = mouse_pos.col;

			mouse_hover.syntax_node_id = get_hovered_node_id(app.as_ref());

			if mouse_hover.syntax_node_id.is_some() || app.current_document().syntax().is_none() {
				// show docs under cursor
				app.hover(mouse_pos.row, mouse_pos.col);
			}
		}
	} else if cursor_coordinates_changed {
		let hovered_node_id = get_hovered_node_id(app.as_ref());
		let hovered_nodes_available = hovered_node_id.is_some() && mouse_hover.syntax_node_id.is_some();
		let cached_node_is_outdated = if hovered_nodes_available {
			mouse_hover.syntax_node_id.unwrap() != hovered_node_id.unwrap()
		} else {
			true
		};

		if cached_node_is_outdated {
			app.hover_close();
			mouse_hover.timer.reset();
		}
	}
}

pub fn mouse_goto_definition(
		key				: Res<ButtonInput<KeyCode>>,
		raypick			: Res<Raypick>,
		q_goto_definition : Query<Entity, With<GotoDefinitionHighlight>>,
		q_word			: Query<(&WordDescription, &WordChildren)>,
		q_transform		: Query<&Transform>,

		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,

	mut color_materials_cache	: ResMut<ColorMaterialsCache>,
	mut material_assets			: ResMut<Assets<StandardMaterial>>,

	mut commands		: Commands,
		app_option		: Option<NonSend<HelixApp>>,
) {
	let Some(app) = app_option else { return };

	if app.should_close() || !app.editor_focused() { return }

	let ctrl_pressed	= key.pressed(KeyCode::ControlLeft) || key.pressed(KeyCode::ControlRight);
	let alt_pressed		= key.pressed(KeyCode::AltLeft) || key.pressed(KeyCode::AltRight);
	let shift_pressed	= key.pressed(KeyCode::ShiftLeft) || key.pressed(KeyCode::ShiftRight);

	let fonts			= ABGlyphFonts::new(&font_assets, &font_handles);
	let row_height		= fonts.main.vertical_advance();
	let column_width	= fonts.main.horizontal_advance_mono();

	let duration_hovered = Duration::from_millis(150);
	let ease_hovered	= EaseFunction::CircularInOut;

	let duration_unhovered = Duration::from_millis(500);
	let ease_unhovered	= EaseFunction::ExponentialOut;

	let hovered_word_entity =
	if let Some(hovered_entity) = raypick.last_hover {
		if q_word.get(hovered_entity).is_ok() {
			Some(hovered_entity)
		} else {
			None
		}
	} else {
		None
	};

	// assign highlight animation on a word that is hovered over
	if let Some(word_entity) = hovered_word_entity {
		let highlight_assigned		= q_goto_definition.get(word_entity).is_ok();
		let (word, word_children)	= q_word.get(word_entity).unwrap();
		let mesh_entity				= word_children.mesh_entity;
		let mesh_transform			= q_transform.get(mesh_entity).unwrap();

		let mut syntax_tree_check	= true;

		let (view, doc) = app.current_ref();
		if let Some(syntax) = doc.syntax() {
			syntax_tree_check		= false;
			let screen_row			= word.row.saturating_sub(app.row_offset_external());
			let char_idx			= view.pos_at_screen_coords(doc, screen_row as u16, word.column as u16, true);

			if let Some(char_idx) = char_idx {
				let text			= doc.text().slice(..);
				let byte			= text.char_to_byte(char_idx);

				let root			= syntax.tree().root_node();
				let hovered_node	= root.descendant_for_byte_range(byte, byte + word.string.len());

				// println!("hovered over node: {:?} {}", hovered_node, text.slice(char_idx .. char_idx + word.string.len()));

				if let Some(node) = hovered_node {
					syntax_tree_check = match node.kind() {
						"identifier"		=> true,
						"type_identifier"	=> true,
						"field_identifier"	=> true,
						_					=> false,
					};
				}
			}
		}

		let highlight_allowed =
		   !highlight_assigned
		&& !alt_pressed
		&& !shift_pressed
		&& !word.is_punctuation
		&& !word.is_numeric
		&& word.is_on_editor
		&& syntax_tree_check
		;

		if ctrl_pressed && highlight_allowed {
			let scale = 1.04;
			let word_width = column_width * word.string.len() as f32;

			let hovered_pos = Vec3::new(
				-(word_width * (scale - 1.0)) / 2.0,
				row_height * (scale - 1.0),
				z_order::surface::text() * scale
			);

			let hovered_scale = mesh_transform.scale * scale;

			let tween = Tween::new(
				ease_hovered,
				duration_hovered,
				TransformLens {
					start : mesh_transform.clone(),
					end : Transform {
						translation : hovered_pos,
						rotation : Quat::from_rotation_y(-3.0f32.to_radians()),
						scale : hovered_scale,
						..default()
					}
				}
			);

			let new_color = word.color.as_rgba_linear() * EMISSIVE_MULTIPLIER_STRONG;

			let material_handle = get_emissive_material_handle(
				new_color,
				&mut color_materials_cache,
				&mut material_assets
			);

			commands.entity(mesh_entity)
				.insert(material_handle.clone_weak())
				.insert(Animator::new(tween))
			;

			commands.entity(word_entity)
				.insert(GotoDefinitionHighlight)
			;
		}
	}

	// remove highlight from words that are no longer hovered over
	for highlighted_word_entity in q_goto_definition.iter() {
		// don't remove highlight from currently hovered word
		if let Some(word_entity) = hovered_word_entity {
			if highlighted_word_entity == word_entity && ctrl_pressed {
				continue;
			}
		}

		// from word entity get collision entity
		let Ok((word, word_children)) = q_word.get(highlighted_word_entity) else { continue };
		let mesh_entity		= word_children.mesh_entity;
		let mesh_transform	= q_transform.get(mesh_entity).unwrap();

		let tween = Tween::new(
			ease_unhovered,
			duration_unhovered,
			TransformLens {
				start	: mesh_transform.clone(),
				end		: Transform::IDENTITY,
			}
		);

		let material_handle = get_color_material_handle(
			word.color,
			&mut color_materials_cache,
			&mut material_assets
		);

		commands.entity(mesh_entity)
			.insert(material_handle.clone_weak())
			.insert(Animator::new(tween))
		;

		commands.entity(highlighted_word_entity)
			.remove::<GotoDefinitionHighlight>()
		;
	}
}

pub fn input_mouse(
	mut mouse_pos_state	: ResMut<MousePosState>,
	mouse_button		: Res<ButtonInput<MouseButton>>,
	mouse_button_state	: Res<MouseButtonState>,
	key					: Res<ButtonInput<KeyCode>>,
	mut cursor_events	: EventReader<CursorMoved>,

	surfaces			: Res<SurfacesMapBevy>,
	font_assets			: Res<Assets<ABGlyphFont>>,
	font_handles		: Res<FontAssetHandles>,

	q_transform			: Query<&GlobalTransform>,
	q_word				: Query<&WordDescription>,
	raypick				: Res<Raypick>,

	bevy_helix_settings	: Res<BevyHelixSettings>,
	dragging_state		: Res<DraggingState>,
	tokio_runtime		: Res<TokioRuntime>,
	app_option			: Option<NonSendMut<HelixApp>>,
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	if dragging_state.is_active() { return }

	let hovered_entity = if let Some(e) = raypick.last_hover { e } else { return };

	let surface_name = if let Ok(word) = q_word.get(hovered_entity) {
		&word.surface_name
	} else {
		// TODO: refactor, if it's a surface we already have the entity to get the transform
		let mut found_surface_name = None;
		for surface in surfaces.iter() {
			if surface.1.entity == hovered_entity {
				found_surface_name = Some(surface.0); // name
			}
		}

		if let Some(name) = found_surface_name {
			name
		} else {
			return
		}
	};

	// getting editor surface first
	let hovered_surface = surfaces.get(surface_name).unwrap();

	// get its transform
	let transform_result = q_transform.get(hovered_surface.entity);
	let surface_transform = if let Ok(transform) = transform_result { transform } else { return };

	// find where mouse cursor is on picked entity
	let cursor_position_world = raypick.ray_pos + raypick.ray_dir * raypick.ray_dist;

	// calculate row and column from surface space cursor coordinates
	let font = font_assets.get(&font_handles.main).unwrap();

	let column_width	= font.horizontal_advance_mono();
	let row_height		= font.vertical_advance();

	// world space to surface space
	let cursor_position_surface = surface_transform.compute_matrix().inverse().transform_point3(cursor_position_world);

	let column			= cursor_position_surface.x / column_width;
	// FIXME: consider bottom anchoring
	let row				= (cursor_position_surface.y.abs() / row_height) - (hovered_surface.scroll_info.offset) as f32;

	// make sure we're in limits of surface area
	let area			= hovered_surface.area;
	if (column < 0.0 || column > area.width as f32) || (row < 0.0 || row > area.height as f32) {
		return
	}

	let modifiers_helix = input::key_code_to_helix_modifiers(&key);

	let column			= column as u16;
	let row				= row as u16;
	let mouse_moved		= !cursor_events.is_empty();

	mouse_pos_state.row = row;
	mouse_pos_state.col = column;
	mouse_pos_state.surface_name = surface_name.clone();

	input::handle_mouse_events(
		&mouse_button,
		&mouse_button_state,
		&modifiers_helix,
		column,
		row,
		mouse_moved,
		&bevy_helix_settings,
		&tokio_runtime,
		&mut app
	);

	cursor_events.clear();
}

// currently not used since scrolling is handled by camera. Keeping it in case we need to pass wheel events to Helix for other reasons in the future
pub fn input_scroll_deprecated(
	mut scroll_events : EventReader<MouseWheel>,
	raypick			: Res<Raypick>,
	surfaces		: Res<SurfacesMapBevy>,

	q_word			: Query<&WordDescription>,
	q_minimap		: Query<&Minimap>,
	q_minimap_viewport : Query<&MinimapViewport>,

	tokio_runtime	: Res<TokioRuntime>,
	app_option		: Option<NonSendMut<HelixApp>>,
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let hover_entity = if let Some(entity) = raypick.last_hover { entity } else { return };

	let surface_editor = surfaces.get(&String::from(EditorView::ID)).unwrap();

	// if mouse is not over editor, minimap or minimap viewport we shouldnt scroll
	if hover_entity != surface_editor.entity
	&& q_minimap.get(hover_entity).is_err()
	&& q_minimap_viewport.get(hover_entity).is_err()
	&& q_word.get(hover_entity).is_err() {
		return
	}

	input::handle_scroll_events(
		&mut scroll_events,
		&tokio_runtime,
		&mut app
	);
}

pub fn input_keyboard(
	mut arrow_keys		: ResMut<ArrowKeysState>,
	mut keyboard_events : EventReader<KeyboardInput>,
		key				: Res<ButtonInput<KeyCode>>,
		bevy_helix_settings : Res<BevyHelixSettings>,
		tokio_runtime	: Res<TokioRuntime>,
		app_option		: Option<NonSendMut<HelixApp>>,
) {
	let Some(mut app) = app_option else { return };

	if app.should_close() { return }

	use KeyCode as KeyCodeBevy;
	use helix_view::keyboard::KeyCode as KeyCodeHelix;

	let now = Instant::now();

	let modifiers_helix = input::key_code_to_helix_modifiers(&key);

	// EventReader<KeyboardInput> gets updated irregularly and if arrow key is being held we want to send updates with more consistent delays between them
	let mut arrow_key_fn = |keycode_bevy: KeyCodeBevy, keycode_helix: KeyCodeHelix, key_press_timing: &mut Option<KeyPressTiming>| {
		let expected_init_press_delay = bevy_helix_settings.key_press_init_delay_seconds;
		let expected_long_press_delay = bevy_helix_settings.key_press_long_delay_seconds;

		if key.just_pressed(keycode_bevy) {
			*key_press_timing = Some(KeyPressTiming{ init: now, long: None });

			input::send_keyboard_event(&keycode_helix, &modifiers_helix, &tokio_runtime, &mut app);

		} else if key.pressed(keycode_bevy) {
			let Some(key_press_timing) = key_press_timing.as_mut() else { panic!("key_press_timing should have been set to Some in key.just_pressed above!") };

			let init_press_time	= key_press_timing.init;
			let long_press_time	= key_press_timing.long;

			let since_init_press = now.duration_since(init_press_time).as_secs_f32();
			let since_long_press = if let Some(long) = long_press_time { now.duration_since(long).as_secs_f32() } else { 0.0 };

			if since_init_press >= expected_init_press_delay && long_press_time.is_none() {
				input::send_keyboard_event(&keycode_helix, &modifiers_helix, &tokio_runtime, &mut app);

				key_press_timing.long = Some(now);
			} else if since_long_press >= expected_long_press_delay && Some(init_press_time) != long_press_time {
				let Some(long_press_time) = long_press_time else { panic!("long_press_time should have been set to Some after expected_init_delay passed!") };
				input::send_keyboard_event(&keycode_helix, &modifiers_helix, &tokio_runtime, &mut app);

				key_press_timing.long = Some(long_press_time + Duration::from_secs_f32(expected_long_press_delay));
			}
		} else if key.just_released(keycode_bevy) {
			*key_press_timing = None;
		}
	};

	arrow_key_fn(KeyCode::ArrowUp,		KeyCodeHelix::Up, 		&mut arrow_keys.last_event_up);
	arrow_key_fn(KeyCode::ArrowDown,	KeyCodeHelix::Down, 	&mut arrow_keys.last_event_down);
	arrow_key_fn(KeyCode::ArrowLeft,	KeyCodeHelix::Left,		&mut arrow_keys.last_event_left);
	arrow_key_fn(KeyCode::ArrowRight,	KeyCodeHelix::Right,	&mut arrow_keys.last_event_right);

	// sending all keyboard events to Helix
	for keyboard_input in keyboard_events.read() {
		match keyboard_input.key_code {
			// ignore up and down arrows because they are processed via ButtonInput<KeyCode>
			KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::ArrowLeft | KeyCode::ArrowRight => continue,

			// ignore ctrl+1..0 as those are used for context switching
			KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4 | KeyCode::Digit5 |
			KeyCode::Digit6 | KeyCode::Digit7 | KeyCode::Digit8 | KeyCode::Digit9 | KeyCode::Digit0
			if key.pressed(KeyCode::ControlLeft) || key.pressed(KeyCode::ControlRight) => continue,
			_ => (),
		}

		if let Some(keycode_helix) = input::keycode_helix_from_bevy(keyboard_input) {
			input::send_keyboard_event(&keycode_helix, &modifiers_helix, &tokio_runtime, &mut app);
		}
	}

	// bevy_helix specific controls

	// inlay hints
	if key.pressed(KeyCode::ControlLeft) && key.just_pressed(KeyCode::AltLeft){
		app.enable_inlay_hints();
	} else if key.just_released(KeyCode::ControlLeft) || key.just_released(KeyCode::AltLeft) {
		app.disable_inlay_hints();
	}
}

pub fn tokio_events(
	app_option		: Option<NonSendMut<HelixApp>>,
	tokio_runtime	: Res<TokioRuntime>,
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	tokio_runtime.block_on(app.handle_tokio_events());
}

pub fn animations_keepalive(
		q_minimap_scroll	: Query<&Animator<MinimapScrollAnimation>>,
	mut framerate_manager	: ResMut<FramerateManager>
) {
	if !q_minimap_scroll.is_empty() {
		let animator = q_minimap_scroll.iter().next().unwrap();
		framerate_manager.request_active_framerate(format!("active MinimapScrollAnimation animator {:.2}%", animator.tweenable().progress() * 100.));
	}
}

// FIXME: dirty solution. we either need a centralized solution like a flag when animator is created to remove itself from entity or do this manually for each case (craaaazy)
pub fn animations_cleanup_components(
		q_minimap_scroll	: Query<(Entity, &Animator<MinimapScrollAnimation>)>,
	mut	commands			: Commands,
) {
	for (e, animator) in q_minimap_scroll.iter() {
		if animator.tweenable().progress() >= 1.0 {
			commands.entity(e).remove::<Animator<MinimapScrollAnimation>>();
		}
	}
}
pub fn on_context_switch_out(
	mut q_visibility	: Query<&mut Visibility>,
		surfaces_bevy	: Res<SurfacesMapBevy>,
) {
	for (_surface_name, surface_bevy) in surfaces_bevy.iter() {
		let mut visibility = q_visibility.get_mut(surface_bevy.entity).unwrap();
		*visibility.as_mut() = Visibility::Hidden;
	}
}

pub fn on_context_switch_in(
	mut q_visibility	: Query<&mut Visibility>,
		surfaces_bevy	: Option<Res<SurfacesMapBevy>>,
	mut q_camera		: Query<&mut ReaderCamera>,
	mut q_window_primary: Query<&mut Window, With<PrimaryWindow>>,
		app_option		: Option<NonSend<HelixApp>>,
) {
	let Some(app) = app_option else { return };
	let Ok(mut camera) = q_camera.get_single_mut() else { return };
	let Ok(mut window) = q_window_primary.get_single_mut() else { return };

	camera.set_all_default_restrictions_false();
	camera.default_enabled_scroll = true;
	camera.apply_default_restrictions();

	camera.row_constant_offset = -1.0; // a cheat for top panel which height is 1 row 

	// somehow even when we specify that on_context_switch_in should be called only in AppMode::Main and after UpdateSecondary
	// it still gets called way too early, probably due to OnEnter(AppContext::TextEditor)
	let Some(surfaces_bevy) = surfaces_bevy else { println!("context_switch_in called without sufaces!"); return };

	for (_surface_name, surface_bevy) in surfaces_bevy.iter() {
		let Ok(mut visibility) = q_visibility.get_mut(surface_bevy.entity) else { continue };
		*visibility.as_mut() = Visibility::Visible;

		if surface_bevy.name == EditorView::ID {
			camera.target_entity = Some(surface_bevy.entity);
		}
	}

	window.title = app.window_title();
}

pub fn update_background_color(
	mut clear_color		: ResMut<ClearColor>,
		app_option		: Option<NonSend<HelixApp>>,
) {
	let Some(app) = app_option else { return };

	if app.should_close() { return }

	let background_style_default = app.editor.theme.get("ui.background");
	let background_color_default = color_from_helix(background_style_default.bg.unwrap_or(HelixColor::Cyan));

	// darken background a little so it would look different from surface background
	let background_color = get_color_as_modified_hsla(background_color_default, 0.0, 0.0, -0.02, 0.0);

	if background_color != clear_color.0 {
		clear_color.0 = background_color
	}
}

pub fn on_window_close_requested(
	mut close_req_events	: EventReader<WindowCloseRequested>,
	mut q_window_primary	: Query<Entity, With<PrimaryWindow>>,
		app_option			: Option<NonSendMut<HelixApp>>,
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let primary_window_entity = q_window_primary.single_mut();

	for e in close_req_events.read() {
		if e.window != primary_window_entity {
			continue;
		}

		app.close();
	}
}

pub fn exit_app(
		app	: Option<NonSend<HelixApp>>,
	mut exit: EventWriter<AppExit>
) {
	if let Some(app) = app {
		if app.should_close() {
			exit.send(AppExit);
		}
	}
}
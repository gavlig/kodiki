use bevy :: prelude	:: *;

use bevy_reader_camera :: ReaderCamera;
use bevy_tweening :: *;

#[cfg(feature = "tracing")]
pub use bevy_puffin :: *;

use helix_term	:: ui		:: EditorView;
use helix_view	:: graphics :: Color as HelixColor;

use futures_lite			:: future;

use super :: *;

use crate :: {
	bevy_framerate_manager :: FramerateManager,
	bevy_helix :: { Matches, MatchesMapCache },
	bevy_ab_glyph :: FontAssetHandles,
	kodiki_ui :: {
		spawn as spawn_common,
		text_cursor :: TextCursor,
		raypick :: *,
	}
};

pub fn update(
	mut	q_minimap			: Query<&mut Minimap>,
	mut	q_minimap_viewport	: Query<&mut MinimapViewport>,
		minimap_render_task : Query<&MinimapRenderTask>,
	 	font_assets			: Res<Assets<ABGlyphFont>>,
	 	font_handles		: Res<FontAssetHandles>,
	mut material_assets		: ResMut<Assets<StandardMaterial>>,
	mut commands			: Commands,
	 	app					: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app { app } else { return };

	if app.should_close() { return }

	// wait until previous async task is done before starting a new one
	if !minimap_render_task.is_empty() { return }

	let mut minimap = q_minimap.single_mut();

	let current_document = app.current_document();
	let current_document_version = current_document.version();

	if let Some(cache) = minimap.document_cache.as_ref() {
		// different document or different color there or different document version are triggering minimap regeneration
		if cache.id == current_document.id()
		&& cache.theme == app.editor.theme.name()
		&& cache.version == current_document_version {
			return;
		}

		// don't update on every version change because it's too expensive, wait for idle timer instead
		if cache.version != current_document_version && cache.id == current_document.id() && !app.idle_timeout_triggered() {
			return;
		}
	}

	minimap.document_cache = Some(SyncDataDoc {
		id			: current_document.id(),
		theme		: app.editor.theme.name().into(),
		version		: current_document_version,
		..default()
	});

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);
	let theme = app.editor.theme.clone();

	// minimap rendering is deferred because rendering big files into texture can get expensive
	minimap.spawn_render_task(&fonts, current_document, theme, app.dark_theme(), &mut commands);
	
	if let Ok(mut viewport) = q_minimap_viewport.get_mut(minimap.viewport_entity) {
		minimap.update_viewport(
			app.dark_theme(),
			&mut viewport,
			&mut material_assets,
			&mut commands
		);
	}
}

pub fn handle_render_tasks(
	mut	q_minimap		: Query<&mut Minimap>,
	mut minimap_render_task : Query<(Entity, &mut MinimapRenderTask)>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut image_assets	: ResMut<Assets<Image>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
    mut commands		: Commands,
	 	app				: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app { app } else { return };

	if app.should_close() { return }

	let mut minimap = q_minimap.single_mut();

	let Ok((entity, mut task)) = minimap_render_task.get_single_mut() else { return };
	if let Some((rows_total, colored_rows, minimap_chunks)) = future::block_on(future::poll_once(&mut task.0)) {
		minimap.apply_render_task_results(
			rows_total,
			colored_rows,
			minimap_chunks,
			&mut mesh_assets,
			&mut image_assets,
			&mut material_assets,
			&mut commands
		);

		commands.entity(entity).remove::<MinimapRenderTask>();

		// these are dependant on minimap size so have to be done after visual counterpart is done
		let current_document = app.current_document();

		minimap.update_bookmarks(
			current_document,
			&app.editor.theme,
			&mut mesh_assets,
			&mut color_materials_cache,
			&mut material_assets,
			&mut commands
		);

		minimap.update_diff_gutter(
			current_document,
			&app.editor.theme,
			&mut mesh_assets,
			&mut color_materials_cache,
			&mut material_assets,
			&mut commands
		);
    }
}

pub fn update_bookmarks(
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
	mut q_minimap		: Query<&mut Minimap>,
	mut commands		: Commands,
		app				: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app { app } else { return };

	if app.should_close() { return }

	let current_document = app.current_document();
	let current_symbols_version = Some(current_document.symbols_version());

	let mut minimap = q_minimap.single_mut();

	if minimap.bookmarks_version == current_symbols_version {
		return;
	}

	minimap.bookmarks_version = current_symbols_version;

	minimap.update_bookmarks(
		current_document,
		&app.editor.theme,
		&mut mesh_assets,
		&mut color_materials_cache,
		&mut material_assets,
		&mut commands
	);
}

pub fn update_diagnostics_highlights(
	mut q_minimap		: Query<&mut Minimap>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
	mut commands		: Commands,
		app_option		: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let doc = app.current_document();

	let mut minimap = q_minimap.single_mut();

	// offsets are calculated with minimap size in mind and if there is an ongoing render task it means the size is going to change
	if minimap.render_task_spawned { return }

	if let Some(cache) = minimap.diagnostics_highlights.cache.as_ref() {
		if cache.doc.id == doc.id()
		&& cache.doc.theme == app.editor.theme.name()
		&& cache.doc.version == doc.version()
		&& cache.diagnostics_version == doc.diagnostics_version() {
			return;
		}
	}

	minimap.diagnostics_highlights.cache = Some(SyncDataDiagnostics {
		doc: SyncDataDoc {
			id			: doc.id(),
			theme		: app.editor.theme.name().into(),
			version		: doc.version(),
			..default()
		},
		diagnostics_version : doc.diagnostics_version()
	});

	minimap.update_diagnostics_highlights(
		doc,
		&app.editor.theme,
		&mut mesh_assets,
		&mut color_materials_cache,
		&mut material_assets,
		&mut commands
	);
}

pub fn update_search_highlights(
		matches_cache	: Res<MatchesMapCache>,
	mut q_minimap		: Query<&mut Minimap>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
	mut commands		: Commands,
		app_option		: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let mut minimap = q_minimap.single_mut();

	// offsets are calculated with minimap size in mind and if there is an ongoing render task it means the size is going to change
	if minimap.render_task_spawned { return }

	let search_kind = SearchKind::Common;
	let search_matches = matches_cache.map.get(&search_kind).unwrap();

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	update_search_highlights_inner(
		search_matches,
		search_kind,
		&mut minimap,
		&mut mesh_assets,
		&mut material_assets,
		&mut color_materials_cache,
		&fonts,
		&mut commands,
		&app
	);
}

pub fn update_selection_search_highlights(
		matches_cache	: Res<MatchesMapCache>,
	mut q_minimap		: Query<&mut Minimap>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
	mut commands		: Commands,
		app_option		: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	if app.active_search_pattern().is_some() { return }
	
	let mut minimap = q_minimap.single_mut();

	// offsets are calculated with minimap size in mind and if there is an ongoing render task it means the size is going to change
	if minimap.render_task_spawned { return }

	let search_kind = SearchKind::Selection;
	let search_matches = matches_cache.map.get(&search_kind).unwrap();

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	update_search_highlights_inner(
		search_matches,
		search_kind,
		&mut minimap,
		&mut mesh_assets,
		&mut material_assets,
		&mut color_materials_cache,
		&fonts,
		&mut commands,
		&app
	);
}

fn update_search_highlights_inner(
	search_matches	: &Matches,
	search_kind		: SearchKind,
	minimap			: &mut Minimap,
	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	color_materials_cache : &mut ColorMaterialsCache,
	fonts			: &ABGlyphFonts,
	commands		: &mut Commands,
	app				: &NonSend<HelixApp>
) {
	if search_matches.is_empty() {
		minimap.despawn_highlights(search_kind.into(), commands);
		return
	}

	let highlights = minimap.get_search_highlights_mut(search_kind);

	if let Some(version_cache) = highlights.cache.as_ref() {
		if *version_cache == search_matches.version {
			return;
		}
	}

	highlights.cache = Some(search_matches.version);

	minimap.update_search_highlights(
		&search_matches.vec,
		search_kind,
		app.current_document(),
		&app.editor.theme,
		fonts,
		mesh_assets,
		color_materials_cache,
		material_assets,
		commands
	);
}

pub fn update_selection_highlights(
	mut q_minimap		: Query<&mut Minimap>,
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
	mut commands		: Commands,
		app				: Option<NonSend<HelixApp>>
) {
	let app = if let Some(app) = app { app } else { return };

	if app.should_close() { return }

	let (view, doc) = app.current_ref();

	let mut minimap = q_minimap.single_mut();

	// offsets are calculated with minimap size in mind and if there is an ongoing render task it means the size is going to change
	if minimap.render_task_spawned { return }

	let current_selection_version = doc.selections_version();

	if let Some(cache) = minimap.selection_highlights.cache.as_ref() {
		if cache.id == doc.id()
		&& cache.theme == app.editor.theme.name()
		&& cache.version == current_selection_version {
			return;
		}
	}

	minimap.selection_highlights.cache = Some(SyncDataDoc {
		id			: doc.id(),
		theme		: app.editor.theme.name().into(),
		version		: current_selection_version,
		..default()
	});

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);
	let selection = doc.selection(view.id);

	minimap.update_selection_highlights(
		doc,
		&selection,
		&app.editor.theme,
		&fonts,
		&mut mesh_assets,
		&mut color_materials_cache,
		&mut material_assets,
		&mut commands
	);
}

pub fn update_transform(
		surfaces_bevy		: Res<SurfacesMapBevy>,
		font_assets			: Res<Assets<ABGlyphFont>>,
		font_handles		: Res<FontAssetHandles>,
		q_minimap			: Query<&Minimap>,
	mut	q_minimap_scaled_mode : Query<&mut MinimapScaledMode>,
		q_minimap_pointer	: Query<&MinimapPointer>,
	mut	q_minimap_viewport	: Query<&mut MinimapViewport>,
		q_camera			: Query<(&ReaderCamera, &Transform)>,
	mut	q_transform_mut		: Query<&mut Transform, Without<ReaderCamera>>,
		q_cursor			: Query<&TextCursor>,
		time				: Res<Time>,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);
	let column_width	= fonts.main.horizontal_advance_mono();
	let row_height		= fonts.main.vertical_advance();

	let (reader_camera, camera_transform) = q_camera.single();

	let surface_editor	= surfaces_bevy.get(&String::from(EditorView::ID)).unwrap();

	let camera_row		= reader_camera.row_offset_in() as f32;

	let minimap			= q_minimap.single();
	let mut minimap_scaled_mode = q_minimap_scaled_mode.single_mut();
	let minimap_scale;

	let visible_minimap_rows = ((reader_camera.visible_rows * row_height) / minimap.row_height).floor() as usize;
	let excess_rows = minimap.rows_total.saturating_sub(visible_minimap_rows);

	{ // minimap transform
		let mut minimap_transform = q_transform_mut.get_mut(minimap.entity).unwrap();
		let mut new_translation = minimap_transform.translation.clone();

		new_translation.x = (surface_editor.area.width as f32 * column_width) + minimap.width() / 2.0 + minimap.padding;
		new_translation.y = camera_transform.translation.y;

		// here and below we calculate y_offset for minimap to either put it on top of viewport
		// or scroll it so that we always see the part that is relevant to cursor position

		// if minimap doesnt fit viewport then scroll it depending on cursor position
		let y_offset_progress_scroll = if excess_rows > 0 {
			let rows_total = minimap.rows_total as f32 - reader_camera.visible_rows;
			let progress = (camera_row / rows_total) * 2.0 - 1.0; // making range from 0..1 to -1..1

			progress * ((excess_rows / 2) as f32) * minimap.row_height
		} else {
			0.0
		};

		// if it fits then put it on top
		let y_offset_full_view = if visible_minimap_rows > minimap.rows_total {
			let gap_offset = (visible_minimap_rows - minimap.rows_total) as f32 / 2.0;
			gap_offset * minimap.row_height
		} else {
			0.0
		};

		// if scaled mode is on we animate minimap scale along with y_offset
		if !minimap_scaled_mode.transition_timer.finished() {
			// first scale/unscale minimap gradually according to transition timer
			minimap_scaled_mode.transition_timer.tick(time.delta());
			let lerp_coef = minimap_scaled_mode.transition_timer.fraction();

			minimap_transform.scale.y = minimap_scaled_mode.scale_from.lerp(minimap_scaled_mode.scale_to, lerp_coef);

			// apply offset vertical offset also gradually according to transition timer
			let (y_offset_from, y_offset_to) = if minimap_scaled_mode.active {
				(y_offset_progress_scroll, y_offset_full_view)
			} else {
				(y_offset_full_view, y_offset_progress_scroll)
			};

			new_translation.y += y_offset_from.lerp(y_offset_to, lerp_coef)
		} else if excess_rows > 0 && !minimap_scaled_mode.active {
			new_translation.y += y_offset_progress_scroll;
		} else {
			new_translation.y += y_offset_full_view;
		};

		minimap_transform.translation = new_translation;

		minimap_scale = minimap_transform.scale;
	}

	{ // cursor pointer transform
		let pointer = q_minimap_pointer.get(minimap.pointer_entity).unwrap();
		let mut pointer_transform = q_transform_mut.get_mut(minimap.pointer_entity).unwrap();

		pointer_transform.translation.x = -minimap.size.x / 2.0 - pointer.size / 2.0;
		pointer_transform.translation.y = (minimap.rows_total as f32 * minimap.row_height) / 2.0;

		if !surface_editor.cursor_entities.is_empty() {
			if let Ok(cursor) = q_cursor.get(surface_editor.cursor_entities[0]) {
				pointer_transform.translation.y -= (cursor.row + 1) as f32 * minimap.row_height;
			}
		}

		// since pointer is rotated we need to rotate scale too. kind of hacky with abs but oh well
		let rotated_scale = pointer_transform.rotation.mul_vec3(minimap_scale).abs();
		pointer_transform.scale = 1.0 / rotated_scale;
	}

	{ // viewport transform + data
		let mut viewport = q_minimap_viewport.get_mut(minimap.viewport_entity).unwrap();
		let mut viewport_transform = q_transform_mut.get_mut(minimap.viewport_entity).unwrap();

		let viewport_height = reader_camera.visible_rows * minimap.row_height;
		viewport_transform.scale.y = viewport_height.max(0.1);

		viewport_transform.translation.y = (minimap.rows_total as f32 * minimap.row_height) / 2.0;
		viewport_transform.translation.y -= camera_row * minimap.row_height + viewport_height / 2.0;

		viewport.current_row = reader_camera.row_offset_out() as usize;
	}
}

pub fn reveal_hovered_bookmark(
	raypick				: Res<Raypick>,
	dragging_state		: Res<DraggingState>,
	q_minimap			: Query<&Minimap>,
	q_transform			: Query<&GlobalTransform>,
	q_bookmark			: Query<&Bookmark, Without<BookmarkRevealed>>,
	q_bookmark_revealed	: Query<&Bookmark, With<BookmarkRevealed>>,
	q_bookmark_hint 	: Query<(Entity, &BookmarkHint)>,

	(
		mut glyph_meshes_cache,
		mut text_meshes_cache,
		mut color_materials_cache,

		mut mesh_assets,
		mut material_assets,
			font_assets,
			font_handles,
	)
	:
	(
		ResMut<GlyphMeshesCache>,
		ResMut<TextMeshesCache>,
		ResMut<ColorMaterialsCache>,

		ResMut<Assets<Mesh>>,
		ResMut<Assets<StandardMaterial>>,
		Res<Assets<ABGlyphFont>>,
		Res<FontAssetHandles>,
	),

		app_option		: Option<NonSend<HelixApp>>,
	mut commands		: Commands
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	if dragging_state.is_active() { return }

	let minimap			= q_minimap.single();
	let minimap_transform = q_transform.get(minimap.entity).unwrap();
	let (minimap_scale, _, _) = minimap_transform.to_scale_rotation_translation();

	let fonts			= ABGlyphFonts::new(&font_assets, &font_handles);
	let font			= fonts.main;
	let row_height		= font.vertical_advance();
	let column_width	= font.horizontal_advance_mono();

	// remove revealed bookmarks that are no longer hovered over
	for (bookmark_hint_entity, bookmark_hint) in q_bookmark_hint.iter() {
		// don't remove currently hovered bookmark hint
		if let Some(hovered_bookmark_entity) = raypick.last_hover {
			if bookmark_hint.owner == hovered_bookmark_entity {
				continue;
			}
		}

		let visual_entity = q_bookmark_revealed.get(bookmark_hint.owner).unwrap().visual_entity;
		bookmark_on_mouse_out(visual_entity, &mut commands);

		commands.entity(bookmark_hint.owner).remove::<BookmarkRevealed>();
		commands.entity(bookmark_hint_entity).despawn_recursive();
	}

	// spawn words(hint) next to hovered symbol bookmarks
	if let Some(hovered_bookmark_entity) = raypick.last_hover {
		let hovered_bookmark = if let Ok(bookmark) = q_bookmark.get(hovered_bookmark_entity) { bookmark } else { return };

		bookmark_on_hover(hovered_bookmark.visual_entity, &mut commands);

		// word

		let text = format!("{:?}: {}", hovered_bookmark.kind, hovered_bookmark.name);

		let (word_mesh_handle, material_handle) = (
			generate_string_mesh_wcache(&text, font, &mut mesh_assets, &mut glyph_meshes_cache, &mut text_meshes_cache),
			get_color_material_handle(
				hovered_bookmark.color,
				&mut color_materials_cache,
				&mut material_assets
			)
		);

		let word_width = column_width * text.len() as f32;
		let word_x = word_width + column_width * 2.0;
		let translation = Vec3::new(-word_x, 0.0, z_order::minimap::bookmark_hint_text());

		let word_mesh_entity = spawn_common::mesh_material_entity_wtranslation(
			&word_mesh_handle,
			&material_handle,
			translation,
			&mut commands
		);

		// background quad

		let background_style = app.editor.theme.get("ui.popup");
		let color = color_from_helix(background_style.bg.unwrap_or_else(|| { HelixColor::Cyan }));

		let background_quad_material_handle = get_color_material_handle(
			color,
			&mut color_materials_cache,
			&mut material_assets
		);

		let quad_width = word_width + column_width;
		let quad_height = row_height * 1.2;
		let quad_position = Vec3::new(
			-word_x + word_width / 2.0,
			row_height / 2.0,
			0.0
		);

		let quad_size = Vec2::new(quad_width, quad_height);
		let quad_mesh_entity = spawn_common::background_quad(
			quad_position,
			quad_size,
			false, /* with_collision */
			Some(&background_quad_material_handle),
			&mut mesh_assets,
			&mut commands
		);

		// holder entity

		let bookmark_hint_entity = commands.spawn((
			VisibilityBundle::default(),
			TransformBundle {
				local : Transform {
					translation : Vec3::Z * z_order::minimap::bookmark_hint(),
					scale : 1.0 / minimap_scale,
					..default()
				},
				..default()
			}
		)).id();

		commands.entity(bookmark_hint_entity)
			.add_child(word_mesh_entity)
			.add_child(quad_mesh_entity)
			.insert(BookmarkHint { owner: hovered_bookmark_entity })
		;

		// final

		commands.entity(hovered_bookmark_entity)
			.add_child(bookmark_hint_entity)
			.insert(BookmarkRevealed)
		;
	}
}

// handle clicking and show hovered line
pub fn input_mouse(
		q_minimap				: Query<&Minimap>,
	mut	q_minimap_scaled_mode	: Query<&mut MinimapScaledMode>,
	mut q_minimap_viewport		: Query<&mut MinimapViewport>,
		q_minimap_hovered_line	: Query<(Entity, &MinimapHoveredLine)>,
		q_transform				: Query<&GlobalTransform>,
		q_camera				: Query<&ReaderCamera>,

		mouse_button	: Res<ButtonInput<MouseButton>>,
		key				: Res<ButtonInput<KeyCode>>,
		raypick			: Res<Raypick>,
	mut dragging_state	: ResMut<DraggingState>,
	mut framerate_manager : ResMut<FramerateManager>,
	mut cursor_events	: EventReader<CursorMoved>,

	(
		mut glyph_meshes_cache,
		mut text_meshes_cache,
		mut color_materials_cache,

		mut mesh_assets,
		mut material_assets,
			font_assets,
			font_handles,
	)
	:
	(
		ResMut<GlyphMeshesCache>,
		ResMut<TextMeshesCache>,
		ResMut<ColorMaterialsCache>,

		ResMut<Assets<Mesh>>,
		ResMut<Assets<StandardMaterial>>,
		Res<Assets<ABGlyphFont>>,
		Res<FontAssetHandles>,
	),

		app_option	: Option<NonSendMut<HelixApp>>,
	mut commands	: Commands
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let animate_viewport_opacity = |viewport_entity: Entity, alpha_start: f32, alpha_end: f32, commands: &mut Commands| {
		let tween = Tween::new(
			EaseFunction::QuadraticInOut,
			Duration::from_millis(150),
			StandardMaterialAlphaLens {
				start : alpha_start,
				end : alpha_end
			}
		);

		commands.entity(viewport_entity).insert(AssetAnimator::new(tween));
	};

	let restore_viewport_opacity = |minimap: &Minimap, viewport: &mut MinimapViewport, commands: &mut Commands| {
		// change viewport opacity to default
		if !viewport.hovered || viewport.dragging_active() {
			return;
		}

		let alpha_from = if viewport.pressed { viewport.alpha_pressed } else { viewport.alpha_hovered };
		animate_viewport_opacity(minimap.viewport_entity, alpha_from, viewport.alpha, commands);

		viewport.hovered = false;
	};

	let despawn_preview_area = |commands: &mut Commands| {
		if let Ok((hovered_line_entity, _)) = q_minimap_hovered_line.get_single() {
			commands.entity(hovered_line_entity).despawn_recursive();
		}
	};

	let try_to_switch_scaled_mode = |scaled_mode: &mut MinimapScaledMode, turning_on_allowed: bool| -> bool {
		if key.just_pressed(KeyCode::ControlLeft) && turning_on_allowed {
			scaled_mode.active = true;
		} else if key.just_released(KeyCode::ControlLeft) {
			scaled_mode.active = false;
		} else {
			return false;
		}

		return true;
	};

	let handle_scaled_mode_animation = |
		minimap			: &Minimap,
		minimap_scale	: Vec3,
		reader_camera	: &ReaderCamera,
		row_height		: f32,
		scaled_mode		: &mut MinimapScaledMode,
	| -> bool {
		let visible_minimap_rows = ((reader_camera.visible_rows * row_height) / minimap.row_height).floor() as usize;
		let excess_rows = minimap.rows_total.saturating_sub(visible_minimap_rows);

		if excess_rows > 0 {
			let y_scale_current		= minimap_scale.y;
			let y_scale_normal		= 1.0;
			let y_scale_squeezed	= visible_minimap_rows as f32 / minimap.rows_total as f32;

			let diff;

			(scaled_mode.scale_from, scaled_mode.scale_to, diff) = if scaled_mode.active {
				(y_scale_current, y_scale_squeezed, (y_scale_current - y_scale_normal).abs())
			} else {
				(y_scale_current, y_scale_normal, (y_scale_squeezed - y_scale_current).abs())
			};

			// since previous transition animation might not be finished yet
			// we need to adjust elapsed time according to y_scale_current
			// so that animation doesnt play the same time for smaller scale change
			let total	= (y_scale_squeezed - y_scale_normal).abs();
			let coef	= diff / total;
			let elapsed	= Duration::from_secs_f32(scaled_mode.transition_timer.duration().as_secs_f32() * coef);

			scaled_mode.transition_timer.reset();
			scaled_mode.transition_timer.set_elapsed(elapsed);
		}

		return true;
	};

	let request_smooth_framerate = |framerate_manager: &mut FramerateManager, reason: String| {
		if !cursor_events.is_empty() || mouse_button.get_pressed().next().is_some() || mouse_button.get_just_pressed().next().is_some() {
			framerate_manager.request_smooth_framerate(reason);
		}
	};

	let request_active_framerate = |framerate_manager: &mut FramerateManager, reason: String| {
		if !cursor_events.is_empty() || mouse_button.get_pressed().next().is_some() || mouse_button.get_just_pressed().next().is_some() {
			framerate_manager.request_active_framerate(reason);
		}
	};

	let on_mouse_out = |minimap: &Minimap, minimap_viewport: &mut MinimapViewport, commands: &mut Commands| {
		despawn_preview_area(commands);
		restore_viewport_opacity(minimap, minimap_viewport, commands);
	};

	let hovered_entity		= if let Some(entity) = raypick.last_hover { entity } else { Entity::from_raw(0) };

	let viewport_probe		= q_minimap_viewport.get_mut(hovered_entity);
	let minimap_probe		= q_minimap.get(hovered_entity);

	// we can work when mouse is hovering over viewport or minimap
	let (mut viewport, minimap) = {
		if viewport_probe.is_ok() {
			let viewport = viewport_probe.unwrap();
			let minimap_entity = viewport.minimap_entity;
			(viewport, q_minimap.get(minimap_entity).unwrap())
		} else if minimap_probe.is_ok() {
			let minimap = minimap_probe.unwrap();
			let viewport_entity = minimap.viewport_entity;
			let mut viewport = q_minimap_viewport.get_mut(viewport_entity).unwrap();

			restore_viewport_opacity(&minimap, &mut viewport, &mut commands);
			(viewport, minimap)
		} else {
			// assuming there is only one viewport, get it to see if it is being dragged
			let viewport = q_minimap_viewport.single_mut();
			let minimap_entity = viewport.minimap_entity;

			(viewport, q_minimap.get(minimap_entity).unwrap())
		}
	};

	let minimap_transform = if let Ok(transform) = q_transform.get(minimap.entity) { transform } else { panic!("input_mouse_minimap: minimap has no transform component!"); };
	let (minimap_scale, _, _) = minimap_transform.to_scale_rotation_translation();

	// find where mouse cursor is on picked entity
	let cursor_position_world = raypick.ray_pos + raypick.ray_dir * raypick.ray_dist;

	// world space to minimap space
	let cursor_position_minimap = minimap_transform.compute_matrix().inverse().transform_point3(cursor_position_world);

	let minimap_row_y	= -(cursor_position_minimap.y - (minimap.size.y / 2.0)).min(0.0); // negating y because row number grows downwards and y grows upwards so we need to make a flip
	let minimap_row		= ((minimap_row_y / minimap.row_height) as usize).min(minimap.colored_rows.len().saturating_sub(1));

	//
    // Mouse can be hovered over: viewport, on minimap but outside viewport, outside minimap completely
    //

	// Viewport
	if hovered_entity == minimap.viewport_entity || viewport.dragging_active() {
		if mouse_button.pressed(MouseButton::Left) {
			if let Some(last_row) = viewport.last_hovered_row {
				let diff = minimap_row as i32 - last_row as i32;
				let new_row = (app.row_offset_internal() as i32 + diff).max(0) as usize;

				app.set_row_offset_internal(new_row);
			}

			viewport.last_hovered_row = Some(minimap_row);

			dragging_state.set_active(hovered_entity);

			if !viewport.pressed {
				animate_viewport_opacity(minimap.viewport_entity, viewport.alpha_hovered, viewport.alpha_pressed, &mut commands);

				viewport.pressed = true;
			}

			request_smooth_framerate(&mut framerate_manager, "mouse button press on minimap viewport".into());
		} else {
			viewport.last_hovered_row = None;

			dragging_state.unset_active();

			if !viewport.hovered || viewport.pressed {
				let alpha_from = if viewport.pressed { viewport.alpha_pressed } else { viewport.alpha };
				animate_viewport_opacity(minimap.viewport_entity, alpha_from, viewport.alpha_hovered, &mut commands);

				viewport.hovered = true;
				viewport.pressed = false;
			}

			request_active_framerate(&mut framerate_manager, "mouse hover on minimap viewport".into());
		}
	// Minimap outside viewport
	} else if hovered_entity == minimap.entity {
		if mouse_button.just_pressed(MouseButton::Left) {
			let reader_camera = q_camera.single();

			// put hovered row in the middle of viewport when clicked
			let scroll_to = minimap_row.saturating_sub((reader_camera.visible_rows / 2.0).floor() as usize);
			minimap.scroll_to_row(
				app.row_offset_internal(),
				scroll_to,
				&mut commands
			);

			minimap.spawn_click_point(
				cursor_position_minimap,
				app.dark_theme(),
				&mut mesh_assets,
				&mut material_assets,
				&mut commands
			);

			request_smooth_framerate(&mut framerate_manager, "mouse button press on minimap".into());
		} else {
			request_active_framerate(&mut framerate_manager, "mouse hover on minimap".into());
		}
	// Outside minimap completely
	} else {
		on_mouse_out(&minimap, &mut viewport, &mut commands);
	}

	//
	// Handle scaled mode: gets triggered when ctrl is held while mouse hovers over minimap
	//

	let mut scaled_mode = q_minimap_scaled_mode.single_mut();

	let scaled_mode_changed = {
		let turning_on_allowed = hovered_entity == minimap.entity || hovered_entity == minimap.viewport_entity;
		try_to_switch_scaled_mode(&mut scaled_mode, turning_on_allowed)
	};

	if scaled_mode_changed {
		let reader_camera = q_camera.single();
		let fonts		= ABGlyphFonts::new(&font_assets, &font_handles);
		let row_height	= fonts.main.vertical_advance();
		handle_scaled_mode_animation(
			minimap,
			minimap_scale,
			reader_camera,
			row_height,
			&mut scaled_mode,
		);
	}

	//
	// Handle preview area: small area with a few lines of code located under mouse cursor rendered in it
	//

	if hovered_entity == minimap.entity && !viewport.dragging_active() && !scaled_mode_changed {
		let fonts	= ABGlyphFonts::new(&font_assets, &font_handles);
		let font	= fonts.main;

		minimap.spawn_preview_area(
			minimap_row,
			minimap_scale,
			q_minimap_hovered_line,
			&font,
			&mut mesh_assets,
			&mut material_assets,
			&mut glyph_meshes_cache,
			&mut text_meshes_cache,
			&mut color_materials_cache,
			app,
			&mut commands
		);
	} else {
		despawn_preview_area(&mut commands);
	}

	cursor_events.clear();
}

pub fn input_mouse_bookmark(
	mouse_button		: Res<ButtonInput<MouseButton>>,
	dragging_state		: Res<DraggingState>,
	q_minimap			: Query<&Minimap>,
	q_symbol_bookmark	: Query<&Bookmark>,
	raypick				: Res<Raypick>,

	app_option			: Option<NonSend<HelixApp>>,
	mut commands		: Commands
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	if dragging_state.is_active() { return }

	if mouse_button.just_pressed(MouseButton::Left) {
		let hover_entity	= if let Some(entity)	= raypick.last_hover 					{ entity } else { return };
		let symbol_bookmark	= if let Ok(symbol)		= q_symbol_bookmark.get(hover_entity)	{ symbol } else { return };

		let target_row		= (symbol_bookmark.location.range.start.line as usize).max(1);
		let minimap			= q_minimap.get(symbol_bookmark.minimap_entity).unwrap();

		minimap.scroll_to_row(
			app.row_offset_internal(),
			target_row,
			&mut commands
		);
	}
}

pub fn update_minimap_scroll_animation(
	mut q_minimap_scroll	: Query<&mut MinimapScrollAnimation>,
		app_option			: Option<NonSendMut<HelixApp>>,
) {
	let mut app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	for mut animation in q_minimap_scroll.iter_mut() {
		if let Some(row) = animation.row_read_to_apply() {
			app.set_row_offset_internal(row);
		}
	}
}

pub fn update_click_point(
	q_minimap_transform	: Query<&Transform, (With<Minimap>, Without<ClickPoint>)>,
	mut q_click_point	: Query<&mut Transform, With<ClickPoint>>,
	mut q_click_point_visual : Query<(Entity, &AssetAnimator<StandardMaterial>), With<ClickPointVisual>>,
	mut commands		: Commands
) {
	let minimap_transform = q_minimap_transform.single();

	for mut transform in q_click_point.iter_mut() {
		// keep click point unscaled to avoid it getting squeezed when minimap scaled mode kicks in
		transform.scale = 1.0 / minimap_transform.scale;
	}

	for (entity, click_point) in q_click_point_visual.iter_mut() {
		// despawn click point entity after animation is done
		if click_point.tweenable().progress() >= 1.0 {
			commands.entity(entity).despawn();
		}
	}
}
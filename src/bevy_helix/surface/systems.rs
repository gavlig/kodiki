use bevy :: prelude :: *;

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines :: *;

use super :: surface :: *;

use crate :: {
	bevy_helix :: spawn,
	kodiki_ui :: {
		spawn as spawn_common,
		text_cursor :: TextCursor
	},
	bevy_ab_glyph :: {
		{ FontAssetHandles, GlyphWithFonts, GlyphMeshesCache, TextMeshesCache, EmojiMaterialsCache },
		glyph_mesh_generator :: generate_string_mesh_wcache,
		emoji_generator :: { generate_emoji_mesh_wcache, generate_emoji_material_wcache },
	},
};

use helix_view :: {
	graphics :: Color as HelixColor,
	document :: Mode,
};

pub fn update(
	app_option			: Option<NonSend<HelixApp>>,
	font_assets			: Res<Assets<ABGlyphFont>>,
	font_handles		: Res<FontAssetHandles>,

		surfaces_helix	: Res<SurfacesMapHelix>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
	mut words_to_spawn	: ResMut<WordsToSpawn>,
	mut quads_to_spawn	: ResMut<ColoringLinesToSpawn>,
	mut despawn			: ResMut<DespawnResource>,
	mut commands        : Commands,

	#[cfg(feature = "debug")]
	mut lines			: ResMut<DebugLines>,
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let row_offset = app.row_offset_external() as i32;

	for (layer_name, surface_helix) in surfaces_helix.iter() {
		let surface_bevy = surfaces_bevy.get_mut(layer_name).unwrap();

		if layer_name == EditorView::ID {
			// update text_description on editor surface for proper camera navigation
			let columns = surface_helix.area.width as usize;
			let rows = app.current_doc_len_lines() as usize;
			surface_bevy.update_text_descriptor(columns, rows, fonts.main, &mut commands);

			// flush all cached rows if active document changed
			if app.active_document_changed {
				surface_bevy.clear_all_rows(&mut despawn);
				surface_bevy.on_scroll_forced(row_offset);
			}
		}

		surface_bevy.update(
			surface_helix,
			row_offset,
			&app.editor.theme,
			&fonts,
			&mut words_to_spawn,
			&mut quads_to_spawn,
			&mut despawn,

			#[cfg(feature = "debug")]
			if layer_name == EditorView::ID { Some(&mut lines) } else { None }
		);
	}
}

pub fn spawn_words(
	(
		mut surfaces_bevy,

		mut glyph_meshes_cache,
		mut text_meshes_cache,
		mut color_materials_cache,
		mut emoji_materials_cache,

		mut mesh_assets,
		mut image_assets,
		mut material_assets,
			font_assets,
			font_handles,
	)
	:
	(
		ResMut<SurfacesMapBevy>,

		ResMut<GlyphMeshesCache>,
		ResMut<TextMeshesCache>,
		ResMut<ColorMaterialsCache>,
		ResMut<EmojiMaterialsCache>,

		ResMut<Assets<Mesh>>,
		ResMut<Assets<Image>>,
		ResMut<Assets<StandardMaterial>>,
		Res<Assets<ABGlyphFont>>,
		Res<FontAssetHandles>,
	),

	mut to_spawn		: ResMut<WordsToSpawn>,

	mut commands		: Commands,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	for (surface_name, surface_bevy) in surfaces_bevy.iter_mut() {
		let words_to_spawn = if let Some(words) = to_spawn.per_surface.get(surface_name) { words } else { continue; };
		let mut surface_children = Vec::new();

		#[cfg(feature = "word_spawn_debug")]
		let (mut spawned_words_log, mut row_index_log) = (String::new(), None);

		for word_desc in words_to_spawn.iter() {
			let color		= word_desc.color;

			let first_char	= word_desc.string.chars().next().unwrap();
			let first_char_string = String::from(first_char);
			let first_symbol = GlyphWithFonts::new(&first_char_string, &fonts);

			// normal glyphs are made of meshes with simple color-material, emojis are made of simple quad mesh with image-material
			let (word_mesh_handle, material_handle) =
			if first_symbol.is_emoji {
				// assert!			(word.string.len() == 1, "for emojis we expect to have 1 word per each emoji! Instead got {} symbols in word [{}]", word.string.len(), word.string);
				(
					generate_emoji_mesh_wcache(&first_symbol, &mut mesh_assets, &mut text_meshes_cache),
					generate_emoji_material_wcache(&first_symbol, &mut image_assets, &mut material_assets, &mut emoji_materials_cache)
				)
			} else {
				(
					generate_string_mesh_wcache(&word_desc.string, first_symbol.current_font(), &mut mesh_assets, &mut glyph_meshes_cache, &mut text_meshes_cache),
					get_color_material_handle(
						color,
						&mut color_materials_cache,
						&mut material_assets
					)
				)
			};

			let word_mesh_entity = spawn_common::mesh_material_entity(
				&word_mesh_handle,
				&material_handle,
				&mut commands
			);

			let word_collision_entity = spawn_common::string_mesh_collision(
				&word_desc.string,
				fonts.main,
				&mut commands
			);

			let word_children = WordChildren {
				mesh_entity : word_mesh_entity,
				collision_entity : word_collision_entity
			};

			let word_entity = spawn::word_entity(word_desc, &word_children, &mut commands);

			let cached_words = &mut surface_bevy.rows[word_desc.cached_row_index].words;
			let cached_word = &mut cached_words[word_desc.word_index];

			cached_word.entity		= Some(word_entity);
			cached_word.mesh_entity	= Some(word_mesh_entity);

			commands.entity(word_entity)
				.add_child(word_mesh_entity)
				.add_child(word_collision_entity)
			;

			// collect word entities to assign them as children to surface afterwards
			surface_children.push(word_entity);

			#[cfg(feature = "word_spawn_debug")] {
				if row_index_log.is_none() || word_desc.row != row_index_log.unwrap() {
					spawned_words_log.push_str(format!("\nrow {} ", word_desc.row).as_str());
					row_index_log = Some(word_desc.row);
				}
				spawned_words_log.push_str(format!("[{}] {} [{:.1} {:.1}] ", word_desc.column, word_desc.string, word_desc.x, word_desc.y).as_str());
			}
		}

		#[cfg(feature = "word_spawn_debug")]
		println!("spawning words for {}\n{}", surface_name, spawned_words_log);

		commands.entity(surface_bevy.entity).push_children(surface_children.as_slice());
	}

	to_spawn.per_surface.clear();
}

pub fn spawn_coloring_lines(
	mut surfaces_bevy		: ResMut<SurfacesMapBevy>,
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
	mut mesh_assets			: ResMut<Assets<Mesh>>,
	mut material_assets		: ResMut<Assets<StandardMaterial>>,
	mut to_spawn			: ResMut<ColoringLinesToSpawn>,
	mut commands			: Commands,
) {
	for (surface_name, surface_bevy) in surfaces_bevy.iter_mut() {
		let lines_to_spawn = if let Some(words) = to_spawn.per_surface.get(surface_name) { words } else { continue; };
		let mut surface_children = Vec::new();

		for line_description in lines_to_spawn.iter() {
			let color		= line_description.color;

			let material_handle = get_color_material_handle(
				color,
				&mut color_materials_cache,
				&mut material_assets
			);

			let line_mesh_entity = spawn_common::background_quad(
				line_description.position(),
				line_description.size(),
				false, /* with_collision */
				Some(&material_handle),
				&mut mesh_assets,
				&mut commands
			);


			commands.entity(line_mesh_entity).insert(line_description.clone());

			let lines_row_bevy = &mut surface_bevy.rows[line_description.cached_row_index].lines;
			lines_row_bevy[line_description.line_index].entity = Some(line_mesh_entity);

			// collect word entities to assign them as children to surface afterwards
			surface_children.push(line_mesh_entity);
		}

		commands.entity(surface_bevy.entity).push_children(surface_children.as_slice());
	}

	to_spawn.per_surface.clear();
}

pub fn highlight_insert_mode(
		mut	surfaces_bevy			: ResMut<SurfacesMapBevy>,
		mut color_materials_cache	: ResMut<ColorMaterialsCache>,
		mut material_assets			: ResMut<Assets<StandardMaterial>>,

	app_option			: Option<NonSend<HelixApp>>,
	mut commands		: Commands,
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let surface_bevy = if let Some(surface) = surfaces_bevy.get_mut(EditorView::ID_STATUSLINE) { surface } else { return };

	assert!(surface_bevy.rows.len() == 1);

	let row = &mut surface_bevy.rows[0];

	if row.words.len() == 0 || row.words[0].mesh_entity.is_none() { return }

	let first_word = &mut row.words[0];

	if app.mode() == Mode::Insert {
		if first_word.string == "INS" && !first_word.is_highlighted {
			commands.entity(first_word.mesh_entity.unwrap())
			.insert(
				get_emissive_material_handle(
					Color::rgb(0.9, 0.1, 0.1) * EMISSIVE_MULTIPLIER_STRONG,
					&mut color_materials_cache,
					&mut material_assets
				)
			);

			first_word.is_highlighted = true;
		}
	} else {
		if first_word.string != "INS" && first_word.is_highlighted {
			commands.entity(first_word.mesh_entity.unwrap())
			.insert(
				get_color_material_handle(
					first_word.color,
					&mut color_materials_cache,
					&mut material_assets
				)
			);

			first_word.is_highlighted = false;
		}
	}
}

pub fn update_diagnostics_highlights(
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
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

	let surface_editor = if let Some(e) = surfaces_bevy.get_mut(EditorView::ID) { e } else { return };

	if let Some(cache) = surface_editor.diagnostics_highlights.cache.as_ref() {
		if cache.doc.id == doc.id()
		&& cache.doc.theme == app.editor.theme.name()
		&& cache.doc.version == doc.version()
		&& (cache.doc.horizontal_offset == Some(view.offset.horizontal_offset) || cache.doc.horizontal_offset.is_none())
		&& cache.diagnostics_version == doc.diagnostics_version() {
			return;
		}
	}

	surface_editor.diagnostics_highlights.cache = Some(SyncDataDiagnostics {
		doc : SyncDataDoc {
			id		: doc.id(),
			theme	: app.editor.theme.name().into(),
			version	: doc.version(),
			horizontal_offset : Some(view.offset.horizontal_offset)
		},
		diagnostics_version : doc.diagnostics_version()
	});

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	surface_editor.update_diagnostics_highlights(
		doc,
		view,
		&app.editor.theme,
		&fonts,
		&mut mesh_assets,
		&mut color_materials_cache,
		&mut material_assets,
		&mut commands
	);
}

pub fn update_search_highlights(
		matches_cache	: Res<MatchesMapCache>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
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

	let surface_editor = if let Some(e) = surfaces_bevy.get_mut(EditorView::ID) { e } else { return };

	let search_kind = SearchKind::Common;
	let search_matches = matches_cache.map.get(&search_kind).unwrap();

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	update_search_highlights_inner(
		search_matches,
		search_kind,
		surface_editor,
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
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
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

	let surface_editor = if let Some(e) = surfaces_bevy.get_mut(EditorView::ID) { e } else { return };

	let search_kind = SearchKind::Selection;
	let search_matches = matches_cache.map.get(&search_kind).unwrap();

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	update_search_highlights_inner(
		search_matches,
		search_kind,
		surface_editor,
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
	surface_editor	: &mut SurfaceBevy,
	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	color_materials_cache : &mut ColorMaterialsCache,
	fonts			: &ABGlyphFonts,
	commands		: &mut Commands,
	app				: &NonSend<HelixApp>
) {
	if search_matches.is_empty() {
		surface_editor.despawn_highlights(search_kind.into(), commands);
		return
	}

	let highlights = surface_editor.get_search_highlights_mut(search_kind);

	if let Some(version_cache) = highlights.cache.as_ref() {
		if *version_cache == search_matches.version {
			return;
		}
	}

	highlights.cache = Some(search_matches.version);

	let (view, doc) = app.current_ref();

	surface_editor.update_search_highlights(
		&search_matches.vec,
		search_kind,
		doc,
		view,
		&app.editor.theme,
		fonts,
		mesh_assets,
		color_materials_cache,
		material_assets,
		commands
	);
}

pub fn update_selection_highlights(
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
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

	let surface_editor = if let Some(e) = surfaces_bevy.get_mut(EditorView::ID) { e } else { return };

	let (view, doc) = app.current_ref();

	let selection_version = doc.selections_version();

	if let Some(cache) = surface_editor.selection_highlights.cache.as_ref() {
		if cache.id == doc.id()
		&& cache.theme == app.editor.theme.name()
		&& cache.version == selection_version
		&& (cache.horizontal_offset == Some(view.offset.horizontal_offset) || cache.horizontal_offset.is_none()) {
			return;
		}
	}

	surface_editor.selection_highlights.cache = Some(SyncDataDoc {
		id			: doc.id(),
		theme		: app.editor.theme.name().into(),
		version		: selection_version,
		horizontal_offset : Some(view.offset.horizontal_offset)
	});

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);
	let selection = doc.selection(view.id);

	surface_editor.update_selection_highlights(
		selection,
		doc,
		view,
		&app.editor.theme,
		&fonts,
		&mut mesh_assets,
		&mut color_materials_cache,
		&mut material_assets,
		&mut commands
	);
}

pub fn update_cursor_highlights(
		q_cursor		: Query<&TextCursor>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
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

	let surface_editor = if let Some(e) = surfaces_bevy.get_mut(EditorView::ID) { e } else { return };

	if surface_editor.cursor_entities.is_empty() { return }

	let cursor = if let Ok(cu) = q_cursor.get(surface_editor.cursor_entities[0]) { cu } else { return };

	if let Some(cursor_cache) = &surface_editor.cursor_highlights.cache {
		if cursor_cache.col == cursor.col && cursor_cache.row == cursor.row {
			return
		}
	}

	surface_editor.cursor_highlights.cache = Some(helix_core::Position{ col: cursor.col, row: cursor.row });

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	surface_editor.update_cursor_highlights(
		cursor.row,
		app.gutter_len(),
		&app.editor.theme,
		&fonts,
		&mut mesh_assets,
		&mut color_materials_cache,
		&mut material_assets,
		&mut commands
	);
}

pub fn update_size(
	mut	surfaces_bevy	: ResMut<SurfacesMapBevy>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
	mut q_bg_quad		: Query<&mut TextBackgroundQuad>,
) {
	let font			= font_assets.get(&font_handles.main).unwrap();
	let row_height		= font.vertical_advance();
	let column_width	= font.horizontal_advance_mono();

	for (_surface_name, surface_bevy) in surfaces_bevy.iter_mut() {
		surface_bevy.size = Vec2::new(
			surface_bevy.area.width as f32 * column_width,
			surface_bevy.area.height as f32 * row_height
		);

		let Ok(mut bg_quad)	= q_bg_quad.get_mut(surface_bevy.bg_quad_entity) else { continue };
		
		bg_quad.columns	= surface_bevy.area.width as usize;
		bg_quad.rows	= surface_bevy.area.height as usize;
	}
}

pub fn update_background_color(
	mut	surfaces_bevy	: ResMut<SurfacesMapBevy>,
	mut q_bg_quad		: Query<&mut TextBackgroundQuad>,
		app_option		: Option<NonSend<HelixApp>>,
) {
	let app = if let Some(app) = app_option { app } else { return };

	if app.should_close() { return }

	let background_style_default = app.editor.theme.get("ui.background");
	let background_color_default = color_from_helix(background_style_default.bg.unwrap_or(HelixColor::Cyan));

	for (_surface_name, surface_bevy) in surfaces_bevy.iter_mut() {
		let Ok(mut bg_quad)	= q_bg_quad.get_mut(surface_bevy.bg_quad_entity) else { continue };

		if bg_quad.color != Some(background_color_default) {
			bg_quad.color = Some(background_color_default);
		}
	}
}

pub fn update_transform(
	mut	surfaces_bevy	: ResMut<SurfacesMapBevy>,
		surfaces_helix	: Res<SurfacesMapHelix>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
		q_camera		: Query<(Entity, &ReaderCamera)>,
	mut	q_transform		: Query<&mut Transform>,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let surface_editor_helix	= surfaces_helix.get(&String::from(EditorView::ID)).unwrap();
	let scroll_offset = {
		let surface_editor_bevy = surfaces_bevy.get(&String::from(EditorView::ID)).unwrap();
		surface_editor_bevy.scroll_info.offset
	};

	let (camera_entity, reader_camera) = q_camera.single();
	let	camera_transform = q_transform.get(camera_entity).unwrap().clone();

	for (surface_name, surface_helix) in surfaces_helix.iter() {
		if surface_name == EditorView::ID {
			continue;
		}

		let surface_bevy = if let Some(surface) = surfaces_bevy.get_mut(surface_name) { surface } else { continue };
		let mut surface_transform = if let Ok(transform) = q_transform.get_mut(surface_bevy.entity)	{ transform } else { continue };

		let area_helix = surface_helix.area;
		let area_bevy = &mut surface_bevy.area;

		let target_pos =
		if surface_helix.placement == SurfacePlacement::AreaCoordinates {
			if area_bevy.same_position(area_helix) {
				continue
			} else {
				area_bevy.assign_position(area_helix);
			}

			SurfaceBevy::calc_attached_position(
				surface_helix.anchor,
				surface_helix.placement,
				surface_helix.area,
				surface_editor_helix.area,
				reader_camera,
				&camera_transform,
				fonts.main,
				scroll_offset,
			)
		} else {
			surface_bevy.attached_position(surface_editor_helix.area, reader_camera, &camera_transform, fonts.main)
		};

		surface_transform.translation = target_pos;
	}
}
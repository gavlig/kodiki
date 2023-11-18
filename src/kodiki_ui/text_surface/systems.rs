use bevy :: prelude :: *;

use super :: *;

use crate :: {
	kodiki_ui :: { spawn as spawn_common, color :: * },
	bevy_ab_glyph :: {
		ABGlyphFont, ABGlyphFonts, FontAssetHandles,
		GlyphWithFonts, GlyphMeshesCache, TextMeshesCache, EmojiMaterialsCache,
		glyph_mesh_generator :: generate_string_mesh_wcache, 
		emoji_generator :: { generate_emoji_mesh_wcache, generate_emoji_material_wcache }
	},
};

// NOTE: maybe use struct with #[derive(SystemParam)] for text generation
pub fn spawn_words(
	(
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

	mut q_text_surface	: Query<(&mut TextSurface, &WordSpawnInfo, Entity)>,
	mut commands		: Commands,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	profile_function!();

	for (mut text_surface, to_spawn, text_surface_entity) in q_text_surface.iter_mut() {
		let mut surface_children = Vec::new();

		for word_coords in to_spawn.word_coords.iter() {
			let row = word_coords.row;
			let word_index = word_coords.index;
			
			// resize can happen
			if row >= text_surface.rows.len() {
				continue;
			}
			
			let word_desc = &mut text_surface.rows[row].words[word_index];

			let color = word_desc.color;

			let first_char = word_desc.string.chars().next().unwrap();
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
					get_color_material_handle(color, &mut color_materials_cache, &mut material_assets)
				)
			};

			// spawning everything we need for word objects
			
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

			let word_entity = commands.spawn((
				TransformBundle {
					local : Transform::from_translation(word_desc.position()),
					..default()
				},
				WordSubEntities {
					mesh_entity : word_mesh_entity,
					collision_entity : word_collision_entity
				},
				VisibilityBundle::default(),
				RaypickHover::default(),
			))
			.id();
			
			commands.entity(word_entity)
				.add_child(word_mesh_entity)
				.add_child(word_collision_entity)
			;
			
			// updating cached entities in TextSurface::rows

			word_desc.entity = Some(word_entity);
			word_desc.mesh_entity = Some(word_mesh_entity);

			// adding WordDescription component to word entity
			commands.entity(word_entity).insert(word_desc.clone());
			
			// collect word entities to assign them as children to surface afterwards
			surface_children.push(word_entity);
		}
		
		for path_desc in to_spawn.paths.iter() {
			let start_index = path_desc.word_chain_start_index;
			let end_index = path_desc.word_chain_end_index;
			let row = path_desc.row_internal;

			let mut word_entities = Vec::new();
			for word_index in start_index ..= end_index {
				let word_desc = &text_surface.rows[row].words[word_index];
				word_entities.push(word_desc.entity.unwrap());
			}

			for word_entity in word_entities.iter() {
				commands.entity(*word_entity).insert(
					PathRowCol {
						file_path	: path_desc.path.as_ref().unwrap().clone(),
						row			: path_desc.row.unwrap_or_default(),
						col			: path_desc.col.unwrap_or_default(),
						entities	: word_entities.clone()
					}
				);
			}
		}

		commands.entity(text_surface_entity)
			.push_children(surface_children.as_slice())
			.remove::<WordSpawnInfo>();
	}
}

pub fn spawn_coloring_lines(
	mut color_materials_cache : ResMut<ColorMaterialsCache>,
	mut mesh_assets			: ResMut<Assets<Mesh>>,
	mut material_assets		: ResMut<Assets<StandardMaterial>>,
	mut q_text_surface		: Query<(&mut TextSurface, &ColoringLinesToSpawn, Entity)>,
	mut commands			: Commands,
) {
	profile_function!();

	for (mut text_surface, lines_to_spawn, text_surface_entity) in q_text_surface.iter_mut() {
		let mut surface_children = Vec::new();

		for line_description in lines_to_spawn.iter() {
			let color		= line_description.color;

			let material_handle = get_color_material_handle(
				color,
				&mut color_materials_cache,
				&mut material_assets
			);

			let quad_mesh_entity = spawn_common::background_quad(
				line_description.position(),
				line_description.size(),
				false, /* with_collision */
				Some(&material_handle),
				&mut mesh_assets,
				&mut commands
			);

			commands.entity(quad_mesh_entity).insert(line_description.clone());

			let cached_lines = &mut text_surface.rows[line_description.row].lines;
			cached_lines[line_description.line_index].entity = Some(quad_mesh_entity);

			// collect word entities to assign them as children to surface afterwards
			surface_children.push(quad_mesh_entity);
		}

		commands.entity(text_surface_entity).push_children(surface_children.as_slice());
		commands.entity(text_surface_entity).remove::<ColoringLinesToSpawn>();
	}
}

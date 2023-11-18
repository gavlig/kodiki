use bevy :: prelude :: *;

use super :: *;

use crate :: {
	kodiki_ui :: { spawn as spawn_kodiki, color :: * },
	bevy_ab_glyph :: {
		ABGlyphFont, ABGlyphFonts, FontAssetHandles, GlyphMeshesCache, TextMeshesCache,
	},
	bevy_framerate_manager :: FramerateManager,
};

fn apply_table_offset(
	transform	: &mut Transform,
	row			: f32,
	col			: f32,
	fonts		: &ABGlyphFonts,
) {
	let column_width	= fonts.main.horizontal_advance_mono();
	let row_height		= fonts.main.vertical_advance();

	transform.translation.x += (column_width * col) * transform.scale.x;
	transform.translation.y -= (row_height * row) * transform.scale.y;
}

pub fn process_string_spawn_requests(
	q_request		: Query<(Entity, &String3dSpawnRequest)>,
	q_attached		: Query<&StringMeshAttached>,

	font_assets		: Res<Assets<ABGlyphFont>>,
	font_handles	: Res<FontAssetHandles>,

	mut text_meshes_cache		: ResMut<TextMeshesCache>,
	mut glyph_meshes_cache		: ResMut<GlyphMeshesCache>,
	mut color_materials_cache	: ResMut<ColorMaterialsCache>,
	mut mesh_assets				: ResMut<Assets<Mesh>>,
	mut material_assets			: ResMut<Assets<StandardMaterial>>,

	mut commands	: Commands
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let column_width = fonts.main.horizontal_advance_mono();
	let row_height = fonts.main.vertical_advance();

	for (requesting_entity, request) in q_request.iter() {
		let mut transform = request.common.transform;

		apply_table_offset(&mut transform, request.common.row, request.common.col, &fonts);

		let string_ref = &request.common.string;

		let string_mesh_entity = spawn_kodiki::string_mesh(
			string_ref,
			request.common.color,
			transform,
			fonts.main,
			&mut mesh_assets,
			&mut material_assets,
			&mut glyph_meshes_cache,
			&mut text_meshes_cache,
			&mut color_materials_cache,
			&mut commands
		);

		commands.entity(requesting_entity)
			.remove::<String3dSpawnRequest>()
			.add_child(string_mesh_entity)
		;

		if let Some(color) = request.common.background_color {
			let material_handle = get_color_material_handle(
				color,
				&mut color_materials_cache,
				&mut material_assets
			);

			let string_len = string_ref.len() as f32; 
			
			let line_pos = Vec3::new((string_len * column_width) / 2.0, row_height / 2.0, 0.0);
			let line_size = Vec2::new(column_width * (string_len + 1.0), row_height); // added 1 for purely cosmetic reasons
			
			let quad_mesh_entity = spawn_kodiki::background_quad(
				line_pos,
				line_size,
				false, /* with_collision */
				Some(&material_handle),
				&mut mesh_assets,
				&mut commands
			);

			commands.entity(string_mesh_entity).add_child(quad_mesh_entity);
		}

		if request.add_attached_component {
			// if there was an attached component already then despawn previous string as emergency measure and complain in logs about it
			if let Ok(attached) = q_attached.get(requesting_entity) {
				let error_msg = format!("StringMeshRequest {} was made for entity that already had this StringMeshAttached: {}", request.common.string, attached.string);
				eprintln!("{error_msg}");
				debug_assert!(false, "{error_msg}");

				commands.entity(attached.entity).despawn_recursive();
			}

			commands.entity(requesting_entity).insert(
				StringMeshAttached {
					id		: request.common.id,
					string	: request.common.string.clone(),
					entity	: string_mesh_entity,
					..default()
				}
			);
		}

		if let Some(callback) = request.callback.as_ref() {
			callback(requesting_entity, string_mesh_entity, &mut commands);
		}
	}
}

pub fn cleanup_string_mesh_attached(
	q_attached		: Query<(Entity, &StringMeshAttached)>,

	mut commands	: Commands
) {
	for (owner_entity, attached) in q_attached.iter() {
		if !attached.despawn_requested {
			continue
		}

		commands.entity(attached.entity).despawn_recursive();
		commands.entity(owner_entity).remove::<StringMeshAttached>();
	}
}

pub fn process_hover_hints(
		raypick			: Res<Raypick>,
	mut q_hint_hover	: Query<&mut HintHover>,
		q_string_mesh_attached : Query<&StringMeshAttached>,
	mut framerate_manager : ResMut<FramerateManager>,
		kodiki_ui		: Res<KodikiUI>,
	mut commands		: Commands
) {
	profile_function!();

	let Some(hovered_entity) = raypick.last_hover else { return };

	let Ok(mut hovered_hint) = q_hint_hover.get_mut(hovered_entity) else { return };

	framerate_manager.request_active_framerate("Mouse hovered over object with HintHover".into());

	if hovered_hint.active { return };

	let hint_text_color = if kodiki_ui.dark_theme { Color::ANTIQUE_WHITE } else { Color::DARK_GRAY };

	let request = String3dSpawnRequest {
		common : CommonString3dSpawnParams {
			id: HintHover::ID,
			color: hint_text_color,
			..hovered_hint.common.clone()
		},
		add_attached_component : true,
		..default()
	};

	if request.add_self_to(hovered_entity, &q_string_mesh_attached, &mut commands) {
		hovered_hint.active = true;
	}
}

pub fn cleanup_hover_hints(
	mut q_hint_hover 			: Query<(Entity, &mut HintHover)>,
	mut	q_string_mesh_attached	: Query<&mut StringMeshAttached>,
		raypick					: Res<Raypick>,
) {
	profile_function!();

	let hovered_entity = raypick.last_hover;

	for (hint_entity, mut hint) in q_hint_hover.iter_mut() {
		// don't remove hint from currently hovered entity
		if hovered_entity.is_some() && hint_entity == hovered_entity.unwrap() {
			continue
		}

		// despawning is deferred so hint will be marked inactive only when it no longer has string mesh attached
		if let Ok(mut attached) = q_string_mesh_attached.get_mut(hint_entity) {
			if attached.id != HintHover::ID {
				continue
			}

			attached.despawn_requested = true;
		} else {
			hint.active = false;
		}
	}
}

// function is not separated into process/cleanup like others because logic allows system to stay compact enough already
pub fn update_hotkey_hints(
	mut q_hint			: Query<(Entity, &mut HintHotkey)>,
	mut q_string_mesh_attached : Query<&mut StringMeshAttached>,
		key				: Res<Input<KeyCode>>,
		kodiki_ui		: Res<KodikiUI>,
	mut commands		: Commands
) {
	let ctrl_pressed = key.pressed(KeyCode::LControl);
	let hint_text_color = if kodiki_ui.dark_theme { Color::ANTIQUE_WHITE } else { Color::DARK_GRAY };

	for (owner_entity, mut hint) in q_hint.iter_mut() {
		if ctrl_pressed {
			if hint.active {
				continue
			}

			let request = String3dSpawnRequest {
				common : CommonString3dSpawnParams {
					id: HintHotkey::ID,
					color: hint_text_color,
					..hint.common.clone()
				},
				add_attached_component : true,
				..default()
			};

			if request.add_self_to_qmut(owner_entity, &q_string_mesh_attached, &mut commands) {
				hint.active = true;
			}
		} else {
			if !hint.active {
				continue
			}

			// despawning is deferred so hint will be marked inactive only when it no longer has string mesh attached
			if let Ok(mut attached) = q_string_mesh_attached.get_mut(owner_entity) {
				if attached.id != HintHotkey::ID {
					continue
				}

				attached.despawn_requested = true;
			} else {
				hint.active = false;
			}
		}
	}
}

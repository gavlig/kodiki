use bevy :: prelude :: *;

#[cfg(feature = "tracing")]
pub use bevy_puffin :: *;

use super :: *;

use crate :: {
	bevy_framerate_manager :: FramerateManager,
	bevy_ab_glyph :: {
		ABGlyphFont, FontAssetHandles,
	},
};

pub fn update_position(
		q_switcher		: Query<(Entity, &ContextSwitcher)>,
		q_children		: Query<&Children>,
		q_camera		: Query<(&ReaderCamera, &Transform)>,
	mut	q_transform_mut	: Query<&mut Transform, Without<ReaderCamera>>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
) {
	profile_function!();

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);
	let row_height = fonts.main.horizontal_advance_mono();

	let (reader_camera, camera_transform) = q_camera.single();

	for (switcher_entity, switcher) in q_switcher.iter() {
		let switcher_children = q_children.get(switcher_entity).unwrap();
		let switcher_children_cnt = switcher_children.len();

		{ // "mothership" switcher entity follows camera on x and y
			let mut transform	= q_transform_mut.get_mut(switcher_entity).unwrap();

			transform.translation.x = camera_transform.translation.x
									- (reader_camera.visible_columns.floor() - 1.0) / 2.0 * row_height					// move to the left border of the window
									+ switcher.width / 2.0
									+ switcher.margin;
			transform.translation.y = camera_transform.translation.y;
		}

		// switcher entries as children of main switcher entity
		for (index, switcher_entry_entity) in switcher_children.iter().enumerate() {
			let mut transform	= q_transform_mut.get_mut(*switcher_entry_entity).unwrap();

			transform.translation.y = (switcher.entry_height * switcher_children_cnt as f32) / 2.0						// move from center to top
									- (switcher.entry_height * index as f32)											// offset for each individual entry from top
									- switcher.margin * (index + 1) as f32;												// margin between entries
		}
	}
}

pub fn update_color(
	mut	q_switcher_entries	: Query<(Entity, &mut ContextSwitcherEntry, &ContextSwitcherGlyph)>,
	mut	color_materials_cache : ResMut<ColorMaterialsCache>,
	mut	material_assets		: ResMut<Assets<StandardMaterial>>,
		kodiki_ui			: Res<KodikiUI>,
	mut commands			: Commands,
) {
	profile_function!();

	for (entity, mut entry, glyph) in q_switcher_entries.iter_mut() {
		let quad_color	= get_color_wmodified_lightness(kodiki_ui.context_switch_color, 0.1);
		if quad_color == entry.quad_color {
			continue;
		}

		// update quad color material

		let quad_material_handle = get_color_material_handle(
			quad_color,
			&mut color_materials_cache,
			&mut material_assets
		);

		commands.entity(entity).insert(quad_material_handle.clone_weak());

		entry.quad_color = quad_color;

		// update glyph color material

		commands.entity(glyph.entity).despawn_recursive();

		let glyph_color = if kodiki_ui.dark_theme { Color::ANTIQUE_WHITE } else { Color::DARK_GRAY };
		commands.entity(entity).insert(
			String3dSpawnRequest {
				common : entry.glyph_spawn_request_color(glyph_color),
				callback : Some(ContextSwitcherEntry::glyph_spawn_callback()),
				..default()
			}
		);
	}
}

pub fn mouse_input(
		raypick			: Res<Raypick>,
		mouse_button	: Res<ButtonInput<MouseButton>>,

	mut q_switcher_entry : Query<&mut ContextSwitcherEntry>,
		q_switcher_highlight : Query<Entity, With<ContextSwitcherHighlight>>,
		q_parent		: Query<&Parent>,
		q_children		: Query<&Children>,
		q_transform		: Query<&Transform>,

	mut framerate_manager		: ResMut<FramerateManager>,
	mut color_materials_cache	: ResMut<ColorMaterialsCache>,
	mut material_assets			: ResMut<Assets<StandardMaterial>>,

	mut commands		: Commands
) {
	profile_function!();

	let hovered_entity = raypick.last_hover;

	let left_button_just_pressed = mouse_button.just_pressed(MouseButton::Left);

	let mut hovered_switcher_entry = if let Some(hovered_entity) = hovered_entity {
		if let Ok(switcher_entry) = q_switcher_entry.get_mut(hovered_entity) {
			Some(switcher_entry)
		} else {
			None
		}
	} else {
		None
	};


	// assign highlight animation switcher that was just hovered over
	// and set it active if it was clicked on
	if let Some(ref mut switcher_entry) = hovered_switcher_entry {
		let hovered_entity = hovered_entity.unwrap();

		let highlight_assigned = q_switcher_highlight.get(hovered_entity).is_ok();
		if !highlight_assigned {
			let quad_transform	= q_transform.get(hovered_entity).unwrap();
			switcher_entry.highlight(
				hovered_entity,
				quad_transform,
				&mut color_materials_cache,
				&mut material_assets,
				&mut commands
			);
		}

		if left_button_just_pressed {
			switcher_entry.is_active = true;
			// is_triggered should be set to false when processed somewhere outside. Maybe make it more strict via some API
			switcher_entry.is_triggered = true;

			if let Some(ref mut callback) = &mut switcher_entry.callback {
			 	callback();
			}
		}

		framerate_manager.request_active_framerate("Mouse hovered over context switcher".into());
	}

	// unset is_active for all other switchers on mouse click event
	if left_button_just_pressed && hovered_entity.is_some() && hovered_switcher_entry.is_some() {
		let hovered_entity = hovered_entity.unwrap();

		let switcher_entity	= q_parent.get(hovered_entity).unwrap();
		let switcher_entries_entities = q_children.get(switcher_entity.get()).unwrap();

		for entry_entity in switcher_entries_entities.iter() {
			if *entry_entity == hovered_entity {
				continue;
			}

			let  mut entry = q_switcher_entry.get_mut(*entry_entity).unwrap();
			entry.is_active = false;
		}
	}
}

pub fn highlights_cleanup(
		q_switcher_highlight	: Query<Entity, With<ContextSwitcherHighlight>>,
		q_switcher_entry 		: Query<&ContextSwitcherEntry>,
		q_transform				: Query<&Transform>,
		raypick					: Res<Raypick>,
	mut color_materials_cache	: ResMut<ColorMaterialsCache>,
	mut material_assets			: ResMut<Assets<StandardMaterial>>,

	mut commands : Commands
) {
	profile_function!();

	let hovered_entity = raypick.last_hover;
	
	// remove highlight from all switchers that are no longer hovered over and not active
	for highlighted_entry_entity in q_switcher_highlight.iter() {
		// don't remove highlight from currently hovered switcher
		if hovered_entity.is_some() && highlighted_entry_entity == hovered_entity.unwrap() {
			continue;
		}

		let Ok(switcher_entry) = q_switcher_entry.get(highlighted_entry_entity) else {
			debug_assert!(false, "Somehow entity with ContextSwitcherHighlight doesnt have ContextSwitcherEntry component!");
			continue;
		};

		// active switcher entries stay highlighted
		if !switcher_entry.is_active {
			let quad_transform = q_transform.get(highlighted_entry_entity).unwrap();

			switcher_entry.unhighlight(
				highlighted_entry_entity,
				quad_transform,
				&mut color_materials_cache,
				&mut material_assets,
				&mut commands
			);
		}
	}
}

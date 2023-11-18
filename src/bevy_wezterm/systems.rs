use bevy :: prelude				:: *;
use bevy :: input :: keyboard	:: *;
use bevy :: input :: mouse		:: *;
use bevy :: window				:: PrimaryWindow;

use bevy_reader_camera			:: ReaderCamera;
use bevy_tweening				:: *;

use termwiz :: {
	cell	:: Intensity,
	surface :: line :: CellRef,
	color	:: { ColorAttribute, SrgbaTuple },
};

use wezterm_portable :: {
	color :: ColorPalette,
	terminalstate :: mouse :: {
		MouseEvent as MouseEventWezTerm,
		MouseButton as MouseButtonWezTerm,
		MouseEventKind as MouseEventKindWezTerm,
	}
};

use super :: *;

use crate :: kodiki :: DespawnResource;
use crate :: kodiki_ui :: {
	ColorMaterialsCache,
	WordSubEntities,
	text_surface :: {
		TextSurface,
		WordDescription, PathRowCol,
		WordsRow, ClusterRowState, WordSpawnInfo,
		ColoringLineRow, ColoringLineRowState, ColoringLinesToSpawn,
		TextSurfaceCellCluster
	},
	color		:: * ,
	tween_lens	:: * ,
	raypick		:: * ,
};
use crate :: bevy_framerate_manager :: FramerateManager;

use std :: time :: Duration;

pub fn update_actions(
	mut	q_terminal : Query<&mut BevyWezTerm>,
	mut framerate_manager : ResMut<FramerateManager>,
) {
	for mut terminal in q_terminal.iter_mut() {
		terminal.perform_actions();

		if terminal.state_changed {
			framerate_manager.request_active_framerate("new WezTerm action".into());
		}
	}
}

fn srgba_to_bevy(srgba: SrgbaTuple) -> Color {
	Color::Rgba { red: srgba.0, green: srgba.1, blue: srgba.2, alpha: srgba.3 }
}

fn color_attribute_to_bevy(color_attribute: ColorAttribute) -> Color {
	match color_attribute {
		ColorAttribute::TrueColorWithPaletteFallback(srgba, _palette_index) => {
			srgba_to_bevy(srgba)
		},
		ColorAttribute::TrueColorWithDefaultFallback(srgba) => {
			srgba_to_bevy(srgba)
		},
		ColorAttribute::PaletteIndex(palette_index) => {
			let srgba = ColorPalette::default_ref().colors.0[palette_index as usize];
			srgba_to_bevy(srgba)
		},
		ColorAttribute::Default => {
			Color::CYAN
		}
	}
}

struct CellClusterRefWrapper<'a> {
	pub cluster: CellRef<'a>,
	pub col: usize,
	pub row: usize,
	pub bold: bool,
}

impl TextSurfaceCellCluster for CellClusterRefWrapper<'_> {
	fn text(&self) -> &str {
		self.cluster.str()
    }

	fn row(&self) -> usize {
        self.row
    }

	fn col(&self) -> usize {
		self.col
    }

	fn foreground(&self) -> Color {
		let srgba = ColorPalette::default_ref().resolve_fg(self.cluster.attrs().foreground(), self.bold);
		srgba_to_bevy(srgba)
    }

	fn background(&self) -> Color {
		let srgba = ColorPalette::default_ref().resolve_bg(self.cluster.attrs().background());
		srgba_to_bevy(srgba)
    }
}

pub fn update_text_surface(
	mut q_terminal_surface	: Query<(&mut BevyWezTerm, &mut TextSurface, Entity)>,
		font_assets			: Res<Assets<ABGlyphFont>>,
		font_handles		: Res<FontAssetHandles>,
	mut entities_to_despawn	: ResMut<DespawnResource>,
	mut commands			: Commands,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	for (mut terminal, mut text_surface, terminal_entity) in q_terminal_surface.iter_mut() {
		let srgba = terminal.wez_state.get_config().color_palette().background;
		let background_color = Color::Rgba { red: srgba.0, green: srgba.1, blue: srgba.2, alpha: srgba.3 };

		let current_seqno = terminal.wez_state.current_seqno();
		let scroll_offset = terminal.wez_state.vertical_scroll_offset();
		let last_rendered_scroll_offset = terminal.last_rendered_scroll_offset;

		let terminal_screen = terminal.wez_state.screen_mut();
		let physical_rows = terminal_screen.physical_rows;

		if physical_rows != text_surface.rows.len() {
			text_surface.on_resize(
				terminal_screen.physical_rows,
				terminal_screen.physical_cols,
				&mut entities_to_despawn
			);
		}

		let mut words_to_spawn = WordSpawnInfo::default();
		let mut lines_to_spawn = ColoringLinesToSpawn::default();

		let mut row_index = 0 as usize;

		let offset_to_visible_lines = terminal_screen.lines.len() - physical_rows - scroll_offset;

        for line in terminal_screen.lines.iter_mut().skip(offset_to_visible_lines) {
			if line.current_seqno() == current_seqno && current_seqno != 0 && scroll_offset == last_rendered_scroll_offset {
				row_index += 1;
				continue
			}

			let mut row_colors_state = ColoringLineRowState::default();
			let mut row_colors = ColoringLineRow::new();

			let mut row_state = ClusterRowState::default();
			let mut row	= WordsRow::new();

			let mut column_index = 0 as usize;

			for cell_wez in line.visible_cells() {
				let cell_attributes = cell_wez.attrs().clone();
				let bold = cell_attributes.intensity() == Intensity::Bold;

				let cluster_width = cell_wez.width();
				let cluster = CellClusterRefWrapper { cluster : cell_wez, col: column_index, row: row_index, bold };

				text_surface.process_cluster_into_row(
					&cluster,
					&background_color,
					&mut row_state,
					&mut row,
					&mut row_colors_state,
					&mut row_colors,
					&fonts
				);

				column_index += cluster_width;
			}

			text_surface.update_cached_row(
				row_index,
				&row,
				&row_colors,
				&mut words_to_spawn,
				&mut lines_to_spawn,
				&mut entities_to_despawn
			);

			line.update_last_change_seqno(current_seqno);

			row_index += 1;
			if row_index >= physical_rows {
				break;
			}
        }

		terminal.last_rendered_scroll_offset = scroll_offset;

		commands.entity(terminal_entity).insert((words_to_spawn, lines_to_spawn));
	}
}

pub fn update_resizer(
	mut	q_terminal_surface	: Query<(Entity, &mut BevyWezTerm, &mut TextSurface)>,
	mut q_resizer			: Query<(&mut Resizer, &mut Transform)>,
	mut q_bg_quad			: Query<&mut TextBackgroundQuad>,
		q_reader_camera		: Query<&ReaderCamera>,
		font_assets			: Res<Assets<ABGlyphFont>>,
		font_handles		: Res<FontAssetHandles>,
	mut entities_to_despawn	: ResMut<DespawnResource>,
	mut commands			: Commands,
) {
	profile_function!();

	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let Ok(camera) = q_reader_camera.get_single() else { return };

	let column_width	= fonts.main.horizontal_advance_mono();
	let row_height		= fonts.main.vertical_advance();

	for (terminal_entity, mut terminal, mut text_surface) in q_terminal_surface.iter_mut() {
		let Ok((mut resizer, mut resizer_transform)) = q_resizer.get_mut(terminal.resizer_entity) else { continue };

		let rows = camera.visible_rows.floor() as usize;
		let cols = resizer.area.x as usize;

		if terminal.resize(rows, cols) {
			if let Some(bg_entity) = text_surface.background_entity {
				if let Ok(mut bg_quad) = q_bg_quad.get_mut(bg_entity) {
					bg_quad.columns	= cols;
					bg_quad.rows	= rows;
				}
			}

			text_surface.on_resize(rows, cols, &mut entities_to_despawn);

			let desc = TextDescriptor {
				rows,
				columns 	: cols,
				glyph_width	: column_width,
				glyph_height: row_height,
			};

			commands.entity(terminal_entity).insert(desc);
		}

		let wez_color		= terminal.wez_state.palette().scrollbar_thumb;
		resizer.quad_color	= Color::RgbaLinear { red: wez_color.0, green: wez_color.1, blue: wez_color.2, alpha: wez_color.3 };

		resizer_transform.translation.x = -resizer.width / 2.0 - resizer.margin;
		resizer_transform.translation.y = -(rows as f32 * row_height) / 2.0;
	}
}

pub fn update_background_color(
		q_terminal_surface	: Query<(&TextSurface, &BevyWezTerm)>,
	mut q_bg_quad			: Query<&mut TextBackgroundQuad>,
) {
	profile_function!();

	for (text_surface, terminal) in q_terminal_surface.iter() {
		let srgba = terminal.wez_state.get_config().color_palette().background;
		let background_color = Color::Rgba { red: srgba.0, green: srgba.1, blue: srgba.2, alpha: srgba.3 };

		if let Some(bg_entity) = text_surface.background_entity {
			if let Ok(mut bg_quad) = q_bg_quad.get_mut(bg_entity) {
				if bg_quad.color != Some(background_color) {
					bg_quad.color = Some(background_color);
				}
			}
		}
	}
}

pub fn update_cursor(
	mut q_cursor 		: Query<(Entity, &mut TextCursor)>,
		q_terminal		: Query<&BevyWezTerm>,
	mut	q_visibility	: Query<(&mut Visibility, &ComputedVisibility)>,
) {
	use termwiz::surface::CursorVisibility;

	for terminal in q_terminal.iter() {
		let Ok((cursor_entity, mut cursor))				= q_cursor.get_mut(terminal.cursor_entity)	else { continue };
		let Ok((mut visibility, computed_visibility))	= q_visibility.get_mut(cursor_entity)		else { continue };

		let cursor_pos_wez = terminal.wez_state.cursor_pos();

		let out_of_range = cursor_pos_wez.y + terminal.wez_state.vertical_scroll_offset() as i64 > terminal.wez_state.screen().physical_rows as i64;
		let wez_visible = cursor_pos_wez.visibility == CursorVisibility::Visible;

		let target_visibility = wez_visible && !out_of_range;

		if computed_visibility.is_visible() != target_visibility {
			*visibility = if target_visibility { Visibility::Inherited } else { Visibility::Hidden };
		}

		cursor.col = cursor_pos_wez.x as usize;
		cursor.row = cursor_pos_wez.y as usize;

		cursor.color = srgba_to_bevy(terminal.wez_state.get_config().color_palette().cursor_bg);
	}
}

pub fn on_context_switch_out(
	mut q_visibility	: Query<&mut Visibility, With<BevyWezTerm>>,
) {
	for mut visibility in q_visibility.iter_mut() {
		*visibility.as_mut() = Visibility::Hidden;
	}
}

pub fn on_context_switch_in(
	mut q_terminal		: Query<(Entity, &BevyWezTerm, &mut Visibility, &mut Transform), Without<ReaderCamera>>,
	mut q_camera		: Query<(&mut ReaderCamera, &Transform), Without<BevyWezTerm>>,
	mut q_window_primary: Query<&mut Window, With<PrimaryWindow>>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
) {
	let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

	let column_width	= fonts.main.horizontal_advance_mono();

	let Ok((mut reader_camera, camera_transform)) = q_camera.get_single_mut() else { return };

	let Ok(mut window) = q_window_primary.get_single_mut() else { return };

	reader_camera.set_all_default_restrictions_false();
	reader_camera.apply_default_restrictions();
	reader_camera.set_row_offset_in(0); // terminal doesnt support scrolling via camera so we need to cleanup this offset
	reader_camera.row_constant_offset = 0.0;

	let mut there_will_be_only_one = false;

	for (terminal_entity, terminal, mut visibility, mut transform) in q_terminal.iter_mut() {
		if !terminal.active {
			continue;
		}

		reader_camera.target_entity = Some(terminal_entity);

		*visibility.as_mut() = Visibility::Visible;

		let x = camera_transform.translation.x + (-column_width * (terminal.wez_state.screen().physical_cols as f32 / 2.0));
		let y = camera_transform.translation.y + reader_camera.y_top; // NOTE: surface anchor is not accounted for
		let z = z_order::surface::base();
		
		transform.translation = Vec3::new(x, y, z);

		window.title = terminal.window_title();

		debug_assert!(!there_will_be_only_one);
		there_will_be_only_one = true;
	}
}

pub fn keyboard(
	mut q_terminal		: Query<&mut BevyWezTerm>,
	mut keyboard_events : EventReader<KeyboardInput>,
		input_key		: Res<Input<KeyCode>>,
) {
	let Ok(mut terminal) = q_terminal.get_single_mut() else { return };

	for keyboard_event in keyboard_events.iter() {
		let _res = terminal.key_up_down(
			keyboard_event,
			&input_key,
			keyboard_event.state.is_pressed()
		);

		// if res.is_err() {
		// 	eprintln!("terminal.key_up_down failed! {}", res.err().unwrap());
		// }
	}
}

pub fn mouse(
		mouse_button	: Res<Input<MouseButton>>,
		input_key		: Res<Input<KeyCode>>,
	mut scroll_events	: EventReader<MouseWheel>,
		cursor_events	: EventReader<CursorMoved>,
	mut q_terminal		: Query<(&mut BevyWezTerm, Entity)>,
		q_transform		: Query<&GlobalTransform>,
		raypick			: Res<Raypick>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
) {
	profile_function!();

	let Ok((mut terminal, terminal_entity)) = q_terminal.get_single_mut() else { return };

	let Ok(surface_transform) = q_transform.get(terminal_entity) else { return };

	// find where mouse cursor is on terminal
	let cursor_position_world = raypick.ray_pos + raypick.ray_dir * raypick.ray_dist;

	// calculate row and column from surface space cursor coordinates
	let font = font_assets.get(&font_handles.main).unwrap();

	let column_width	= font.horizontal_advance_mono();
	let row_height		= font.vertical_advance();

	// world space to surface space
	let cursor_position_surface = surface_transform.compute_matrix().inverse().transform_point3(cursor_position_world);

	let column			= cursor_position_surface.x / column_width;
	let row				= cursor_position_surface.y.abs() / row_height + 1.0; // + 1 was found empirically

	let modifiers		= BevyWezTerm::key_modifiers_bevy_to_wez(&input_key);

	let mut send_mouse_event = |mouse_event_kind: MouseEventKindWezTerm, mouse_button: MouseButtonWezTerm| {
		let mouse_event = MouseEventWezTerm {
			x			: column as usize,
			y			: row as i64,
			kind		: mouse_event_kind,
			button		: mouse_button,
			modifiers,
			x_pixel_offset : 0,
			y_pixel_offset : 0,
		};

		let res = terminal.wez_state.mouse_event(mouse_event);

		match res {
			Err(e) => { eprintln!("bevy_wezterm: failed mouse event {:?} {:?} {}", mouse_event_kind, mouse_button, e) },
			_ => (),
		}
	};

	let bevy2wezterm_mouse_button = |mouse_button_in: &MouseButton| -> Option<MouseButtonWezTerm> {
		match mouse_button_in {
			MouseButton::Left	=> Some(MouseButtonWezTerm::Left),
			MouseButton::Right	=> Some(MouseButtonWezTerm::Right),
			MouseButton::Middle => Some(MouseButtonWezTerm::Middle),
			_ => None
		}
	};

	//

	for just_pressed in mouse_button.get_just_pressed() {
		let Some(mouse_button) = bevy2wezterm_mouse_button(just_pressed) else {
			eprintln!("bevy_wezterm: unrecognized just_pressed mouse input: {:?}", just_pressed);
			continue;
		};
		send_mouse_event(MouseEventKindWezTerm::Press, mouse_button);
	}

	for just_released in mouse_button.get_just_released() {
		let Some(mouse_button) = bevy2wezterm_mouse_button(just_released) else {
			eprintln!("bevy_wezterm: unrecognized just_released mouse input: {:?}", just_released);
			continue;
		};
		send_mouse_event(MouseEventKindWezTerm::Release, mouse_button);
	}

	if !cursor_events.is_empty() {
		send_mouse_event(MouseEventKindWezTerm::Move, MouseButtonWezTerm::None);
	}

	let mut line_accumulator = Vec2::ZERO;
	let mut pixel_accumulator = Vec2::ZERO;

	let scroll_multiplier = 3.0;

	for scroll_event in scroll_events.iter() {
		let x = scroll_event.x;
		let y = scroll_event.y;

		match scroll_event.unit {
			MouseScrollUnit::Line => {
				line_accumulator.x += if x != 0.0 { x.signum() * scroll_multiplier } else { 0.0 };
				line_accumulator.y += if y != 0.0 { y.signum() * scroll_multiplier } else { 0.0 };
			},
			MouseScrollUnit::Pixel => {
				pixel_accumulator += Vec2::new(scroll_event.x, scroll_event.y)
			},
		}
	}

	// line

	if line_accumulator.y > 0.0 {
		let wheel_up = line_accumulator.y as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelUp(wheel_up));
	}

	if line_accumulator.y < 0.0 {
		let wheel_down = line_accumulator.y.abs() as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelDown(wheel_down));
	}

	if line_accumulator.x > 0.0 {
		let wheel_right = line_accumulator.x as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelRight(wheel_right));
	}

	if line_accumulator.x < 0.0 {
		let wheel_left = line_accumulator.x.abs() as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelLeft(wheel_left));
	}

	// // pixel

	if pixel_accumulator.y > 0.0 {
		let wheel_up = (pixel_accumulator.y / 20.) as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelUp(wheel_up));
	}

	if pixel_accumulator.y < 0.0 {
		let wheel_down = (pixel_accumulator.y.abs() / 20.) as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelDown(wheel_down));
	}

	if pixel_accumulator.x > 0.0 {
		let wheel_right = (pixel_accumulator.x / 20.) as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelRight(wheel_right));
	}

	if pixel_accumulator.x < 0.0 {
		let wheel_left = (pixel_accumulator.x.abs() / 20.) as usize;
		send_mouse_event(MouseEventKindWezTerm::Press, MouseButtonWezTerm::WheelLeft(wheel_left));
	}
}

pub fn mouse_goto_path(
		mouse_button	: Res<Input<MouseButton>>,
		key				: Res<Input<KeyCode>>,
		raypick			: Res<Raypick>,
		q_highlight		: Query<Entity, With<GotoPathHighlight>>,
		q_word			: Query<(&WordDescription, &WordSubEntities)>,
		q_path			: Query<&PathRowCol>,
		q_transform		: Query<&Transform>,

		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,

	mut color_materials_cache	: ResMut<ColorMaterialsCache>,
	mut material_assets			: ResMut<Assets<StandardMaterial>>,

	mut commands		: Commands,
) {
	profile_function!();

	let ctrl_pressed	= key.pressed(KeyCode::LControl);
	let alt_pressed		= key.pressed(KeyCode::LAlt);
	let shift_pressed	= key.pressed(KeyCode::LShift);

	let fonts			= ABGlyphFonts::new(&font_assets, &font_handles);
	let row_height		= fonts.main.vertical_advance();

	let duration_hovered = Duration::from_millis(150);
	let ease_hovered	= EaseFunction::CircularInOut;

	let duration_unhovered = Duration::from_millis(500);
	let ease_unhovered	= EaseFunction::ExponentialOut;

	let hovered_path = if let Some(hovered_entity) = raypick.last_hover {
		if let Ok(path) = q_path.get(hovered_entity) {
			Some(path)
		} else {
			None
		}
	} else {
		None
	};

	// handle hovering over a word entity with path component assigned to it
	if let Some(path) = hovered_path {
		let hovered_entity		= raypick.last_hover.unwrap();
		let highlight_assigned	= q_highlight.get(hovered_entity).is_ok();
		let highlight_allowed	= !highlight_assigned && !alt_pressed && !shift_pressed;

		// assign highlight animation on a path that is hovered over
		if ctrl_pressed && highlight_allowed {
			for word_entity in path.entities.iter() {
				let (word, word_children)	= q_word.get(*word_entity).unwrap();
				let mesh_entity				= word_children.mesh_entity;
				let mesh_transform			= q_transform.get(mesh_entity).unwrap();

				let scale = 1.07;

				let hovered_pos = Vec3::new(
					0.0,
					row_height * (scale - 1.0),
					z_order::surface::text() * 2.0
				);

				let hovered_scale = mesh_transform.scale * Vec3::new(1.0, scale, 1.0);

				let tween = Tween::new(
					ease_hovered,
					duration_hovered,
					TransformLens {
						start : mesh_transform.clone(),
						end : Transform {
							translation : hovered_pos,
							scale : hovered_scale,
							..default()
						}
					}
				);

				let new_color = word.color.as_rgba_linear() * EMISSIVE_MULTIPLIER_MEDIUM;

				let material_handle = get_emissive_material_handle(
					new_color,
					&mut color_materials_cache,
					&mut material_assets
				);

				commands.entity(mesh_entity)
					.insert(material_handle.clone_weak())
					.insert(Animator::new(tween))
				;

				commands.entity(*word_entity)
					.insert(GotoPathHighlight)
				;
			}
		}

		// mark hovered word containing path component as clicked for further processing outside this system
		if mouse_button.just_pressed(MouseButton::Left) {
			commands.entity(hovered_entity).insert(Clicked);
		}
	}

	// remove highlight from words that are no longer hovered over
	for highlighted_word_entity in q_highlight.iter() {
		// don't remove highlight from currently hovered word
		if let Some(path) = hovered_path {
			if path.entities.contains(&highlighted_word_entity) && ctrl_pressed {
				continue
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
			.remove::<GotoPathHighlight>()
		;
	}
}

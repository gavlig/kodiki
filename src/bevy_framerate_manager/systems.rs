use bevy :: prelude				:: *;
use bevy :: input :: mouse		:: *;
use bevy :: winit				:: WinitSettings;

use bevy_framepace				:: { FramepaceSettings, Limiter };
use bevy_reader_camera			:: ReaderCamera;
use bevy_tweening				:: *;

use bevy_vfx_bag :: post_processing :: masks :: Mask;

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines	:: *;

#[cfg(feature = "tracing")]
pub use bevy_puffin :: *;

use super :: { FramerateMode, FramerateManager, FramerateIndicator, FramerateDebug };

use crate :: {
	z_order,
	kodiki_ui :: { *, raypick :: *, color :: * },
	kodiki :: DespawnResource,
};

// Limit framerate when possible to avoid using up all cpu all the time
pub fn update(
	mut	q_camera			: Query<(&mut ReaderCamera, &Transform)>,
	mut manager				: ResMut<FramerateManager>,
	mut framepace_settings	: ResMut<FramepaceSettings>,
	mut winit_settings		: ResMut<WinitSettings>,
		time				: Res<Time>,
		mouse_button		: Res<Input<MouseButton>>,
		mouse_wheel_events	: EventReader<MouseWheel>,
		key					: Res<Input<KeyCode>>,
) {
	let Ok((mut camera, camera_transform)) = q_camera.get_single_mut() else { return };

	manager.clear_on_next_entry();

	// let any_key_pressed = key.get_pressed().next().is_some();

	let mut any_key_pressed = false;
	for press in key.get_pressed() {
		manager.log(format!("pressed: {:?}", press));
		any_key_pressed = true;
	}

	let any_scroll_event = !mouse_wheel_events.is_empty();
	let any_mouse_button_pressed = mouse_button.get_pressed().next().is_some();

	manager.clear_internal_state();
	manager.set_input_state(any_key_pressed, any_mouse_button_pressed, any_scroll_event);
	manager.set_camera_state(camera.move_requested(), camera.is_zooming(), camera.is_moving(&camera_transform));

	let camera_potentially_active = manager.camera_potentially_active();

	let current_frame_duration = match manager.mode() {
		FramerateMode::Idle => {
			match winit_settings.focused_mode {
				bevy::winit::UpdateMode::Continuous | bevy::winit::UpdateMode::Reactive { .. } => {
					panic!("in FramerateMode::Idle winit_settings.focused_mode should only be UpdateMode::ReactiveLowPower!");
				},
				bevy::winit::UpdateMode::ReactiveLowPower { max_wait, .. } => max_wait
			}
		},
		_ => {
			match framepace_settings.limiter {
				Limiter::Off | Limiter::Auto => {
					let frame_duration = manager.set_mode_and_get_duration(FramerateMode::Smooth);
					framepace_settings.limiter = Limiter::Manual(frame_duration);
					// no need to process anything if limiter wasn't manual since this state wasnt set by us
					return;
				}

				Limiter::Manual(frame_duration) => frame_duration,
			}
		}
	};

	let mut new_frame_duration = None;

	let smooth_framerate_condition = manager.read_and_apply_smooth_framerate_condition();
	let active_framerate_condition = manager.read_and_apply_active_framerate_condition();

	// put to idle if no action or reset idle timer otherwise
	if smooth_framerate_condition || active_framerate_condition {
		manager.reset_idle_timer();
	} else {
		manager.tick_idle_timer(&time);

		let idle_timer_finished = manager.idle_timer_finished();

		if !idle_timer_finished && current_frame_duration != manager.active_frame_duration() {
			manager.log("limiting fps to active while waiting for idle timer".into());

			new_frame_duration = Some(
				manager.set_mode_and_get_duration(FramerateMode::Active)
			)
		} else if idle_timer_finished && current_frame_duration != manager.idle_frame_duration() {
			manager.log("limiting fps to idle".into());

			new_frame_duration = Some(
				manager.set_mode_and_get_duration(FramerateMode::Idle)
			)
		}
	}

	// camera is potentially active -> wake it up
	if camera_potentially_active && camera.is_dormant() && manager.animations_allowed(&time) {
		manager.log("camera just started moving -> waking it up".into());

		camera.wake_up();
	} else if !camera_potentially_active && camera.is_awake() && manager.idle_timer.finished() {
		manager.log("camera inactive long enough -> putting to sleep".into());

		camera.put_to_sleep();
	}

	// setting new limiter if conditions are met and it isn't already set
	if smooth_framerate_condition && current_frame_duration != manager.smooth_frame_duration() {
		manager.log("smooth framerate condition -> unlimited fps".into());

		new_frame_duration = Some(
			manager.set_mode_and_get_duration(FramerateMode::Smooth)
		)
	}
	else
	if active_framerate_condition && current_frame_duration != manager.active_frame_duration() && !smooth_framerate_condition {
		manager.log("active framerate condition -> limiting fps to 60".into());

		new_frame_duration = Some(
			manager.set_mode_and_get_duration(FramerateMode::Active)
		)
	}

	if let Some(new_frame_duration) = new_frame_duration {
		match manager.mode() {
			// for idle mode we use bevy's native power management since it handles low fps much better than bevy_framepace (no random crashes without callstack)
			// and allows user interactions to trigger instant updates instead of waiting for idle timer to finish
			FramerateMode::Idle => {
				winit_settings.focused_mode = bevy::winit::UpdateMode::ReactiveLowPower {
					max_wait: new_frame_duration,
					ignore_cursor_movement: true
				};

				framepace_settings.limiter = Limiter::Off;
			},
			// for other modes we use bevy_framepace since it handles high fps much better than native bevy ReactiveLowPower (frame durations are way more consistent)
			_ => {
				winit_settings.focused_mode = bevy::winit::UpdateMode::Continuous;

				framepace_settings.limiter = Limiter::Manual(new_frame_duration);
			}
		}
	}
}

use crate :: bevy_ab_glyph :: {
	ABGlyphFont, ABGlyphFonts, FontAssetHandles,
};

pub fn visualize(
	mut framerate_debug	: ResMut<FramerateDebug>,
		framerate_manager : Res<FramerateManager>,
		q_camera		: Query<(Entity, &ReaderCamera, &Transform)>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
	mut	mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut	color_materials_cache : ResMut<ColorMaterialsCache>,
		time			: Res<Time>,
	mut despawn			: ResMut<DespawnResource>,
	mut commands		: Commands,
) {
	let Ok((camera_entity, reader_camera, camera_transform)) = q_camera.get_single() else { return };

	let x = reader_camera.x_left;
	let y = reader_camera.y_top;
	let z = -camera_transform.translation.z + z_order::surface::last();
	
	let framerate_color = framerate_manager.current_framerate_color();
	let dot_radius	= FramerateIndicator::default().radius;
	let dot_diameter = dot_radius * 2.0;

	let dot_translation = Vec3::new(x + dot_diameter, y - dot_diameter, z);

	let dot_entity = match framerate_debug.dot_entity {
		Some(entity) => entity,
		None => {
			let translation = dot_translation;
			let entity = FramerateDebug::spawn_framerate_dot(
				dot_radius,
				framerate_color,
				translation,
				&mut mesh_assets,
				&mut material_assets,
				&mut color_materials_cache,
				&mut commands
			);

			commands.entity(camera_entity).add_child(entity);

			framerate_debug.dot_entity = Some(entity);

			entity
		}
	};

	let translation_changed = !framerate_debug.dot_translation.abs_diff_eq(dot_translation, f32::EPSILON);
	let color_changed = framerate_debug.dot_color != framerate_color;
	if color_changed || translation_changed {
		let updated_material_handle = get_color_material_handle(
			framerate_color,
			&mut color_materials_cache,
			&mut material_assets
		);

		let updated_transform = Transform::from_translation(dot_translation);

		commands.entity(dot_entity).insert((
			updated_material_handle,
			updated_transform
		));

		framerate_debug.dot_color = framerate_color;
		framerate_debug.dot_translation = dot_translation;
	}

	if framerate_debug.extra_info_enabled {
		let mut logs = Vec::new();

		FramerateDebug::collect_state_logs(&framerate_manager, &time, &mut logs);
		FramerateDebug::collect_extra_logs(&framerate_manager, &mut logs);

		if framerate_debug.lines != logs {
			if let Some(entity) = framerate_debug.lines_entity {
				despawn.recursive.push(entity);	
			}

			let fonts = ABGlyphFonts::new(&font_assets, &font_handles);

			let column_width = fonts.main.horizontal_advance_mono();
			let row_height = fonts.main.vertical_advance();

			let y = dot_translation.y - FramerateIndicator::default().radius * 3.0;
			let lines_translation = Vec3::new(x, y, z);
		
			let lines_entity = FramerateDebug::spawn_visualization_logs(
				lines_translation,
				row_height,
				column_width,
				&logs,
				&mut commands
			);

			commands.entity(camera_entity).add_child(lines_entity);
			
			framerate_debug.lines_entity = Some(lines_entity);

			framerate_debug.lines = logs;
		}
	}
}

pub fn animations_keepalive(
		q_transform			: Query<&Animator<Transform>>,
		q_standard_material	: Query<&AssetAnimator<StandardMaterial>>,
		q_mask				: Query<&Animator<Mask>>,
	mut framerate_manager	: ResMut<FramerateManager>
) {
	if !q_transform.is_empty() {
		let animator = q_transform.iter().next().unwrap();
		framerate_manager.request_active_framerate(format!("active Transform animator {:.2}%", animator.tweenable().progress() * 100.));
	}

	if !q_standard_material.is_empty() {
		let animator = q_standard_material.iter().next().unwrap();
		framerate_manager.request_active_framerate(format!("active StandardMaterial animator {:.2}%", animator.tweenable().progress() * 100.));
	}

	if !q_mask.is_empty() {
		let animator = q_mask.iter().next().unwrap();
		framerate_manager.request_active_framerate(format!("active Mask animator {:.2}%", animator.tweenable().progress() * 100.));
	}
}

// FIXME: dirty solution. we either need a centralized solution like a flag when animator is created to remove itself from entity or do this manually for each case (craaaazy)
pub fn animations_cleanup_components(
		q_transform			: Query<(Entity, &Animator<Transform>)>,
		q_standard_material	: Query<(Entity, &AssetAnimator<StandardMaterial>)>,
		// q_mask				: Query<(Entity, &Animator<Mask>)>,
	mut	commands			: Commands,
) {
	for (e, animator) in q_transform.iter() {
		if animator.tweenable().progress() >= 1.0 {
			commands.entity(e).remove::<Animator<Transform>>();
		}
	}

	for (e, animator) in q_standard_material.iter() {
		if animator.tweenable().progress() >= 1.0 {
			commands.entity(e).remove::<AssetAnimator<StandardMaterial>>();
		}
	}

	// we currently have only one mask: for insert mode and it is taken care of in helix_mode_tween_events
	// for (e, animator) in q_mask.iter() {
	// 	if animator.tweenable().progress() >= 1.0 {
	// 		commands.entity(e).remove::<Animator<Mask>>();
	// 	}
	// }
}

pub fn mouse_input(
		raypick			: Res<Raypick>,
		mouse_button	: Res<Input<MouseButton>>,
	mut q_framerate_indicator : Query<(Entity, &mut FramerateIndicator)>,
		q_transform		: Query<&Transform>,

	mut framerate_debug	: ResMut<FramerateDebug>,

	mut despawn			: ResMut<DespawnResource>,
	mut commands		: Commands
) {
	profile_function!();

	let hovered_entity = raypick.last_hover;

	let left_button_just_pressed = mouse_button.just_pressed(MouseButton::Left);

	let hovered_indicator = if let Some(entity) = hovered_entity {
		if let Ok((_, indicator)) = q_framerate_indicator.get_mut(entity) {
			Some(indicator)
		} else {
			None
		}
	} else {
		None
	};

	if let Some(mut indicator) = hovered_indicator {
		let hovered_entity = hovered_entity.unwrap();
		let transform = q_transform.get(hovered_entity).unwrap();
		indicator.highlight(hovered_entity, transform, &mut commands);

		if left_button_just_pressed {
			indicator.click(hovered_entity, transform, &mut commands);

			framerate_debug.extra_info_enabled = !framerate_debug.extra_info_enabled;

			// cleanup extra logs when they are turned off
			if !framerate_debug.extra_info_enabled && framerate_debug.lines_entity.is_some() {
				despawn.recursive.push(framerate_debug.lines_entity.unwrap());
				framerate_debug.lines_entity = None;
			}
		}
	} else if let Ok((indicator_entity, mut indicator)) = q_framerate_indicator.get_single_mut() {
		let transform = q_transform.get(indicator_entity).unwrap();
		indicator.unhighlight(
			indicator_entity,
			transform,
			&mut commands
		);
	}
}

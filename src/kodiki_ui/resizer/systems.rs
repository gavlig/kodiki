use bevy :: prelude		:: *;
use bevy_tweening		:: *;

#[cfg(feature = "tracing")]
pub use bevy_puffin		:: *;

use super				:: *;

use crate :: kodiki_ui	:: { * , color :: * , tween_lens :: * , raypick :: * };

use std :: time :: Duration;

pub fn input_mouse(
	mut	q_resizer		: Query<&mut Resizer>,
		mouse_button	: Res<Input<MouseButton>>,
		raypick			: Res<Raypick>,
	mut	dragging_state	: ResMut<DraggingState>,
) {
	profile_function!();

	let (mut resizer, resizer_entity) = {
		let hovered_entity = raypick.last_hover.unwrap_or(Entity::from_raw(0));
		let resizer_probe = q_resizer.get_mut(hovered_entity);
		
		if let Ok(resizer) = resizer_probe {
			(resizer, hovered_entity)
		} else if dragging_state.is_active() {
			let dragging_entity = dragging_state.entity.unwrap();

			let Ok(resizer) = q_resizer.get_mut(dragging_entity) else { return };

			// resizer has to be "dragging_active", there is a logic error if it's not at this point
			// since we got the resizer from q_resizer from entity stored in dragging_state
			if !resizer.dragging_active() {
				debug_assert!(false); 
				return
			}

			(resizer, dragging_entity)
		} else {
			return
		}
	};

	if mouse_button.just_pressed(MouseButton::Left) {
		resizer.init_mouse_pos	= Some(raypick.mouse_pos);
		resizer.init_area		= Some(resizer.area);

		dragging_state.set_active(resizer_entity);
	} else if mouse_button.pressed(MouseButton::Left) {
		let init_mouse_pos	= if let Some(c) = resizer.init_mouse_pos { c } else { error!("resizer::input_mouse: init_cursor_pos is None though mouse is not just pressed!"); return };
		let init_area		= if let Some(a) = resizer.init_area { a }		else { error!("resizer::input_mouse: init_area is None though mouse is not just pressed!"); return };

		let column_width = init_mouse_pos.x.abs() / (init_area.x / 2) as f32;

		let mouse_pos_diff = raypick.mouse_pos - init_mouse_pos;

		let diff_columns = (mouse_pos_diff.x / column_width).abs() as u32 * 2;

		let mut new_area = init_area.clone();
		new_area.x = if mouse_pos_diff.x < 0.0 {
			new_area.x.saturating_add(diff_columns)
		} else {
			new_area.x.saturating_sub(diff_columns)
		}.max(40);

		resizer.area = new_area;
	} else {
		resizer.init_mouse_pos	= None;
		resizer.init_area		= None;
		dragging_state.unset_active();
	}
}

pub fn update_color(
	mut	q_resizer			: Query<&mut Resizer>,
	mut	color_materials_cache : ResMut<ColorMaterialsCache>,
	mut	material_assets		: ResMut<Assets<StandardMaterial>>,
	mut commands			: Commands,
) {
	profile_function!();

	for mut resizer in q_resizer.iter_mut() {
		if resizer.quad_color_cached == resizer.quad_color {
			continue;
		}

		let circles_color = get_color_wmodified_lightness(resizer.quad_color, 0.1);

		let quad_material_handle = get_color_material_handle(
			resizer.quad_color,
			&mut color_materials_cache,
			&mut material_assets
		);

		let circles_material_handle = get_color_material_handle(
			circles_color,
			&mut color_materials_cache,
			&mut material_assets
		);

		commands.entity(resizer.quad_entity).insert(quad_material_handle.clone_weak());
		commands.entity(resizer.circles_entity).insert(circles_material_handle.clone_weak());

		resizer.quad_color_cached = resizer.quad_color;
		resizer.circles_color = circles_color;
	}
}


pub fn highlight_hovered(
		raypick			: Res<Raypick>,
		q_hover_highlight : Query<Entity, With<ResizerHighlight>>,
		q_resizer		: Query<&Resizer>,
		q_transform		: Query<&Transform>,

	mut color_materials_cache	: ResMut<ColorMaterialsCache>,
	mut material_assets			: ResMut<Assets<StandardMaterial>>,

	mut commands		: Commands
) {
	profile_function!();

	let duration_hovered = Duration::from_millis(150);
	let ease_hovered	= EaseFunction::CircularInOut;

	let duration_unhovered = Duration::from_millis(500);
	let ease_unhovered	= EaseFunction::ExponentialOut;

	let hovered_entity		= if let Some(entity) = raypick.last_hover { entity } else { Entity::from_raw(0) };

	let resizer_probe		= q_resizer.get(hovered_entity);

	// get resizer from hovered entity or remove highlight from it
	let resizer = if let Ok(r) = resizer_probe { r } else {
		for resizer in q_resizer.iter() {
			if resizer.dragging_active() {
				continue
			}

			// remove highlight from resizer that is no longer hovered over
			if q_hover_highlight.get(resizer.entity).is_err() {
				continue
			}

			let quad_transform = q_transform.get(resizer.quad_entity).unwrap();

			let tween = Tween::new(
				ease_unhovered,
				duration_unhovered,
				TransformLens {
					start	: quad_transform.clone(),
					end		: Transform::IDENTITY,
				}
			);

			let quad_material_handle = get_color_material_handle(
				resizer.quad_color,
				&mut color_materials_cache,
				&mut material_assets
			);

			let circles_material_handle = get_color_material_handle(
				resizer.circles_color,
				&mut color_materials_cache,
				&mut material_assets
			);

			commands.entity(resizer.entity).remove::<ResizerHighlight>();
			commands.entity(resizer.quad_entity)
				.insert(Animator::new(tween))
				.insert(quad_material_handle.clone_weak())
			;

			commands.entity(resizer.circles_entity).insert(circles_material_handle.clone_weak());
		}

		return
	};

	// assign highlight animation on resizer that is hovered over
	let highlight_assigned		= q_hover_highlight.get(resizer.entity).is_ok();
	if highlight_assigned {
		return
	}

	let quad_transform			= q_transform.get(resizer.quad_entity).unwrap();

	let scale = 1.07;

	let hovered_scale = Vec3::new(quad_transform.scale.x * scale, quad_transform.scale.y, quad_transform.scale.z);

	let tween = Tween::new(
		ease_hovered,
		duration_hovered,
		TransformLens {
			start : quad_transform.clone(),
			end : Transform {
				translation : quad_transform.translation,
				scale : hovered_scale,
				..default()
			}
		}
	);

	let new_quad_color = get_color_wmodified_lightness(resizer.quad_color, 0.1);
	let new_circles_color = get_color_wmodified_lightness(resizer.circles_color, -0.1);

	let quad_material_handle = get_color_material_handle(
		new_quad_color,
		&mut color_materials_cache,
		&mut material_assets
	);

	let circles_material_handle = get_color_material_handle(
		new_circles_color,
		&mut color_materials_cache,
		&mut material_assets
	);

	commands.entity(resizer.entity).insert(ResizerHighlight);
	commands.entity(resizer.quad_entity)
		.insert(Animator::new(tween))
		.insert(quad_material_handle.clone_weak())
	;

	commands.entity(resizer.circles_entity).insert(circles_material_handle.clone_weak());
}
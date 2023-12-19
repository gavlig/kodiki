use bevy :: { prelude :: * , window :: PrimaryWindow };
use bevy_reader_camera :: *;
use bevy_rapier3d :: prelude :: *;

use super :: { * , systems_utils :: * };

pub fn cast_raypick(
		rapier_context	: Res<RapierContext>,
	mut	raypick			: ResMut<Raypick>,
		q_camera		: Query<(&Camera, &GlobalTransform), With<ReaderCamera>>,
	mut	q_hover			: Query<&mut RaypickHover>,
		q_parent		: Query<&Parent>,
		q_visibility	: Query<&ViewVisibility>,
		q_window_primary : Query<&Window, With<PrimaryWindow>>,
) {
	let window = if let Ok(w) = q_window_primary.get_single() { w } else { return; };

	let Ok((camera, camera_transform)) = q_camera.get_single() else { return };

	// First, compute a ray from the mouse position.
	let (mouse_pos, ray_pos, ray_dir) = ray_from_mouse_position(window, camera, camera_transform);

	// Unhover previously hovered entity if it still exists
	if let Some(last_hover_entity) = raypick.last_hover {
		if let Ok(mut hover) = q_hover.get_mut(last_hover_entity) {
			hover.set_hovered(false);
		}
	}

	// Prepare callback for raycast to filter out invisible entities
	let mut hovered_entity = None;
	let mut ray_dist = Raypick::default().ray_dist;

	let raycast_callback = |entity: Entity, intersection: RayIntersection| -> bool {
		let mut hit_entity = entity;

		if let Ok(mut hover) = q_hover.get_mut(hit_entity) {
			if !hover.hovered() {
				hover.set_hovered(true);
			}
		} else {
			// try look up parents with Hover component
			let mut hover_parent_found = false;

			while !hover_parent_found {
				// return if no parent because we don't consider entities without Hover component
				let Ok(parent) = q_parent.get(hit_entity) else { return true };

				hit_entity = parent.get();
				if let Ok(mut hover) = q_hover.get_mut(hit_entity) {
					if !hover.hovered() {
						hover.set_hovered(true);
					}
					hover_parent_found = true;
				}
			}
		}

		// we want to avoid hitting invisible entities so check Visibility

		let visible = if let Ok(visibility) = q_visibility.get(hit_entity) {
			visibility.get()
		} else {
			// entities with Hover but without Visibility are likely children of a complex entity with visible part separated
			// so we want to keep finding them
			true
		};

		if visible && intersection.toi < ray_dist {
			ray_dist = intersection.toi;
			hovered_entity = Some(hit_entity);
		}

		// returning false means we do an early out of the raycast as in we found what we were looking for
		// but since order of calls is random we need to process every result
		true
	};

	// Finally cast the ray. results will be in hovered_entity and ray_dist
	rapier_context.intersections_with_ray(
		ray_pos,
		ray_dir,
		Raypick::default().ray_dist,	// max_toi == ray length
		true,							// solid
		QueryFilter::new(),
		raycast_callback
	);

	if hovered_entity.is_none() {
		ray_dist = raypick.ray_dist; // keep reusing last obtained ray_dist for consistency in calculations (minimap viewport dragging for example)
	}

	*raypick = Raypick {
		ray_pos,
		ray_dir,
		ray_dist,
		mouse_pos,
		last_hover : hovered_entity
	};
}
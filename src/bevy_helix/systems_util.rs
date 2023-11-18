use bevy :: prelude :: *;
use bevy_tweening :: *;

#[cfg(feature = "stats")]
use bevy_debug_text_overlay :: screen_print;

use bevy_reader_camera :: ReaderCamera;

use super :: {
	TweenPoint,
	helix_app :: HelixApp,
	surface	:: *,
};

use crate :: {
	z_order,
	kodiki :: DespawnResource,
	bevy_ab_glyph :: ABGlyphFont
};

use helix_term	:: ui			:: EditorView;
use helix_view	:: graphics		:: Color as HelixColor;

use std :: time :: Duration;

pub fn spawn_new_surfaces(
	surfaces_helix		: &mut SurfacesMapHelix,
	surfaces_bevy		: &mut SurfacesMapBevy,

	reader_camera		: &ReaderCamera,
	camera_transform	: &Transform,
	font				: &ABGlyphFont,

	mut mesh_assets		: &mut Assets<Mesh>,
	mut commands		: &mut Commands,
) {
	let scroll_offset = {
		let surface_bevy_editor = surfaces_bevy.get(&String::from(EditorView::ID)).unwrap();
		surface_bevy_editor.scroll_info.offset
	};

	let surface_helix_editor = surfaces_helix.get(&String::from(EditorView::ID)).unwrap();

	for (surface_name, surface_helix) in surfaces_helix.iter() {
		if surfaces_bevy.contains_key(surface_name) {
			continue;
		}

		let target_pos	= SurfaceBevy::calc_attached_position(
			surface_helix.anchor,
			surface_helix.placement,
			surface_helix.area,
			surface_helix_editor.area,
			reader_camera,
			camera_transform,
			font,
			scroll_offset,
		);

		let animate_spawning = false; // surface_helix.placement != SurfacePlacement::AreaCoordinates;

		let start_pos = if animate_spawning {
			let y = camera_transform.translation.y - reader_camera.y_top;
			let z = z_order::surface::child_surface();
			Vec3::new(0.0, y, z)
		} else {
			target_pos
		};

		let surface_bevy = SurfaceBevy::spawn(
			surface_name,
			Some(start_pos),
			false,	/* editor */
			false,	/* scroll_enabled */
			None,	/* resizer_entity */
			&surface_helix,
			&font,

			&mut mesh_assets,
			&mut commands
		);

		if animate_spawning {
			let tween_point = TweenPoint {
				pos: target_pos,
				ease_function: EaseFunction::ExponentialOut,
				delay: Duration::from_millis(250),
			};

			surface_bevy.animate(
				start_pos,
				Vec::from([tween_point]),
				commands
			);
		}

		surfaces_bevy.insert(surface_name.clone(), surface_bevy);
	}
}

pub fn despawn_unused_surfaces(
	surfaces_helix	: &mut SurfacesMapHelix,
	surfaces_bevy	: &mut SurfacesMapBevy,
	despawn			: &mut DespawnResource
) {
    let mut to_remove = Vec::<String>::default();

	// surfaces helix
    for (surface_name, surface_helix) in surfaces_helix.iter_mut() {
		// if "dirty" is false it means that during render surface wasn't modified/filled up, meaning it's not longer used
		if surface_helix.dirty() {
			continue;
		}

		to_remove.push(surface_name.clone());
	}
    for layer in to_remove.iter() {
		surfaces_helix.remove(layer);
	}

	// surfaces bevy
    for (layer_name, surface_bevy) in surfaces_bevy.iter_mut() {
		if surfaces_helix.contains_key(layer_name) {
			continue;
		}

		despawn.recursive.push(surface_bevy.entity);

		to_remove.push(layer_name.clone());
	}
    for layer in to_remove {
		surfaces_bevy.remove(&layer);
	}
}

pub fn benchmark_surface_render(surfaces_bevy: &mut SurfacesMapBevy, surfaces_helix: &mut SurfacesMapHelix, app: &HelixApp) {
    let surface_bevy_editor = surfaces_bevy.get_mut(&String::from(EditorView::ID)).unwrap();
    if surface_bevy_editor.update {
		let surface_helix_editor = surfaces_helix.get_mut(&String::from(EditorView::ID)).unwrap();
		for cell in surface_helix_editor.content.iter_mut() {
			cell.symbol = String::from("A");
			cell.bg = app.editor.theme.get("ui.background").bg.unwrap_or_else(|| { HelixColor::Black });
		}
	}
}

#[cfg(feature = "stats")]
pub fn screen_print_active_layers(
	surfaces_helix : &SurfacesMapHelix,
) {
	let mut surface_names_str = String::default();
	surface_names_str.push_str(format!("{} helix layers:\n", surfaces_helix.len()).as_str());
	for (name, surface) in surfaces_helix.iter() {
		surface_names_str.push_str(" - ");
		surface_names_str.push_str(format!("{} len: {} w: {} h: {}", name, surface.content.len(), surface.area.width, surface.area.height).as_str());
		surface_names_str.push('\n');
	}
	screen_print!("\n{}", surface_names_str);
}

#[cfg(feature = "stats")]
pub fn screen_print_stats(
	surfaces_bevy : &SurfacesMapBevy,
) {
	let mut stats	= String::default();
	stats.push_str	("stats:\n");

	let mut words_cnt = 0;
	for (_name, surface) in surfaces_bevy.iter() {
		for row in surface.rows.iter() {
			words_cnt += row.words.len();
		}
	}
	stats.push_str(format!("words: {}", words_cnt).as_str());
	screen_print!("\n{}", stats);
}
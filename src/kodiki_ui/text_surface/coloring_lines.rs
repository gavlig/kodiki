use bevy :: prelude :: *;

use super :: *;

#[derive(Default)]
pub struct RowState {
	pub line_started	: bool,
	pub synced			: bool,
	pub ended			: bool,
}

pub fn append_cluster_to_row<'a>(
	cluster				: &impl TextSurfaceCellCluster,
	background_color	: &Color,
	new_row				: &mut ColoringLineRow,
	row_state			: &mut RowState,
	surface_name		: &String,
	surface_coords		: &TextSurfaceCoords,
	fonts				: &'a ABGlyphFonts<'a>,
) {
	let line_color		= cluster.background();
	let is_space		= line_color == *background_color;

    if row_state.line_started {
		let last_line	= new_row.last_mut().unwrap();

		let different_color = line_color != last_line.color;
		let line_ended	= is_space || different_color;

		if !line_ended {
			last_line.length += 1;
		} else {
			row_state.line_started = false;
		}
	}

    if !is_space && !row_state.line_started {
		row_state.line_started = true;

		let font		= fonts.main;
		let v_advance	= font.vertical_advance();
		let h_advance	= font.horizontal_advance_mono();

		let new_line = ColoringLineDescription {
			color			: line_color,
			row				: surface_coords.row,
			column			: surface_coords.column,
			line_index		: new_row.len(),
			x				: surface_coords.x,
			y				: surface_coords.y,
			glyph_width		: h_advance,
			height			: v_advance,
			length			: 1,
			surface_name	: surface_name.clone(),
			..default()
		};

		new_row.push(new_line);
	}
}

pub fn update_cached_row(
	cached_row	: &mut ColoringLineRow,
	new_row		: &ColoringLineRow,
	to_spawn	: &mut ColoringLinesToSpawn,
	to_despawn	: &mut DespawnResource,
) {
	let cached_row_len	= cached_row.len();

	for (new_line_index, new_line) in new_row.iter().enumerate() {
		if new_line_index >= cached_row_len {
			cached_row.push(new_line.clone());
			to_spawn.push(new_line.clone());
			continue;
		}

		let cached_line = &cached_row[new_line_index];
		if line_has_changed(new_line, cached_line) || cached_line.entity.is_none() {
			let line_was_despawned = cached_row[new_line_index].entity.is_none();

			if !line_was_despawned {
				let outdated	= &cached_row[new_line_index];
				to_despawn.recursive.push(outdated.entity.unwrap());
			}

			cached_row[new_line_index] = new_line.clone();
			to_spawn.push(new_line.clone());
		}
	}

	let new_row_len = new_row.len();
    if new_row_len == 0 || new_row_len < cached_row_len {
		cleanup_cached_row_from(new_row_len, cached_row, to_despawn);
	}

}

fn line_has_changed(
	new_line	: &ColoringLineDescription,
	cached_line	: &ColoringLineDescription,
) -> bool {
	let same_line =
		cached_line.length == new_line.length &&
		cached_line.column == new_line.column &&
		cached_line.color == new_line.color;

	return !same_line;
}

fn cleanup_cached_row_from(
	cleanup_index_from	: usize,
	cached_row			: &mut ColoringLineRow,
	despawn				: &mut DespawnResource
) {
	let cached_row_len	= cached_row.len();

	for i in cleanup_index_from .. cached_row_len {
		let cached_line = &mut cached_row[i];
		if let Some(entity) = cached_line.entity {
			despawn.recursive.push(entity);
			cached_line.entity = None;
		}
	}

	assert!(cleanup_index_from <= cached_row_len);
	cached_row.truncate(cleanup_index_from);
}

use std :: path		:: { PathBuf };

use bevy			:: { prelude :: * };
use bevy_mod_picking:: { * };

pub fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}

pub fn check_selection_recursive(
	children	: &Children,
	q_children	: &Query<&Children>,
	q_selection : &Query<&Selection>,
	depth		: u32,
	max_depth 	: u32
 ) -> bool {
	let mut selection_found = false;
	for child in children.iter() {
		let selection = match q_selection.get(*child) {
			Ok(s) => s,
			Err(_) => continue,
		};

		if selection.selected() {
			selection_found = true;
		} else {
			if depth >= max_depth {
				continue;
			}
			let subchildren = q_children.get(*child);
			if subchildren.is_ok() {
				selection_found = check_selection_recursive(subchildren.unwrap(), q_children, q_selection, depth + 1, max_depth);
			}
		}

		if selection_found {
			break;
		}
	}

	selection_found
}
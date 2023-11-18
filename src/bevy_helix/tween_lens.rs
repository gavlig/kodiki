use bevy_tweening :: *;

use super :: minimap :: MinimapScrollAnimation;

/// A lens to manipulate current_row field of [`Minimap`] component.
///
/// [`Minimap`]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MinimapRowLens {
	/// Start row.
	pub start: usize,
	/// End row.
	pub end: usize,
}

impl Lens<MinimapScrollAnimation> for MinimapRowLens {
	fn lerp(&mut self, target: &mut MinimapScrollAnimation, ratio: f32) {
		// lerp is not implemented for usize so using u32 as workaround
		let v0 = self.start as u32;
		let v1 = self.end as u32;
		let value = v0.lerp(&v1, &ratio) as usize;
		target.set_row(value);
	}
}

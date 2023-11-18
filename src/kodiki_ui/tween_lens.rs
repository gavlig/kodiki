use bevy :: prelude :: *;
use bevy_tweening	:: *;
use bevy_vfx_bag :: post_processing :: masks :: Mask;

/// A lens to manipulate the whole [`Transform`] component.
///
/// [`Transform`]: https://docs.rs/bevy/0.9.0/bevy/transform/components/struct.Transform.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformLens {
	/// Start value of the transform.
	pub start: Transform,
	/// End value of the transform.
	pub end: Transform,
}

impl Lens<Transform> for TransformLens {
	fn lerp(&mut self, target: &mut Transform, ratio: f32) {
		let value = self.start.translation + (self.end.translation - self.start.translation) * ratio;
		target.translation = value;

		target.rotation = self.start.rotation.slerp(self.end.rotation, ratio);

		let value = self.start.scale + (self.end.scale - self.start.scale) * ratio;
		target.scale = value;
	}
}

/// A lens to manipulate StandardMaterial's alpha
///
/// [`StandardMaterial`]: https://docs.rs/bevy/0.9.0/bevy/pbr/struct.StandardMaterial.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StandardMaterialAlphaLens {
    /// Start alpha.
    pub start: f32,
    /// End alpha.
    pub end: f32,
}

impl Lens<StandardMaterial> for StandardMaterialAlphaLens {
    fn lerp(&mut self, target: &mut StandardMaterial, ratio: f32) {
        let value = self.start.lerp(&self.end, &ratio);
		target.base_color.set_a(value);
    }
}

/// A lens to manipulate strength field of [`Mask`] component.
///
/// [`Mask`]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MaskLens {
	pub start_strength	: f32,
	pub end_strength	: f32,
	pub start_fade		: f32,
	pub end_fade		: f32,
}

impl Lens<Mask> for MaskLens {
	fn lerp(&mut self, target: &mut Mask, ratio: f32) {
		let v0 = self.start_strength;
		let v1 = self.end_strength;
		let value = v0.lerp(&v1, &ratio);
		target.strength = value;

		let v0 = self.start_fade;
		let v1 = self.end_fade;
		let value = v0.lerp(&v1, &ratio);
		target.fade = value;
	}
}

#[derive(Component)]
pub struct MaskFadingIn;

#[derive(Component)]
pub struct MaskFadingOut;
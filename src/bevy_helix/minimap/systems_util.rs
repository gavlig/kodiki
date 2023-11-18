use bevy :: prelude :: *;

use bevy_tweening :: { *, lens :: TransformScaleLens };

use std :: time :: Duration;

pub fn bookmark_on_mouse_out(
	entity		: Entity,
	commands	: &mut Commands
) {
	animate_scale(entity, true, Duration::from_millis(200), commands);
}

pub fn bookmark_on_hover(
	entity		: Entity,
	commands	: &mut Commands
) {
	animate_scale(entity, false, Duration::from_millis(100), commands);
}

fn animate_scale(
	entity		: Entity,
	reverse		: bool,
	duration	: Duration,
	commands	: &mut Commands
) {
	let (start, end) = if !reverse {
		(Vec3::ONE, Vec3::ONE * 1.5)
	} else {
		(Vec3::ONE * 1.5, Vec3::ONE)
	};

	let tween = Tween::new(
		EaseFunction::QuadraticInOut,
		duration,
		TransformScaleLens { start, end }
	);

	commands.entity(entity).insert(Animator::new(tween));
}
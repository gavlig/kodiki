use bevy				:: prelude :: { * };
use bevy_tweening		:: { * };

use std :: time :: Duration;

pub struct TweenPoint {
	pub pos			: Vec3,
	pub delay		: Duration,
	pub ease_function : EaseFunction,
}
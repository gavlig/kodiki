use bevy :: prelude :: *;
use super :: *;

pub fn animations_allowed(
	time				: Res<Time>,
	framerate_manager	: Res<FramerateManager>
) -> bool {
	// don't animate anything in idle state and don't start playing it until framerate has become better than idle
	let condition_met = framerate_manager.animations_allowed(&time);

	condition_met
}


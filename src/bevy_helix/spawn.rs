use bevy :: prelude :: *;
use super :: surface :: *;
use crate :: kodiki_ui :: raypick :: RaypickHover;

pub fn word_entity(
	word_description: &WordDescription,
	word_children	: &WordChildren,
	commands		: &mut Commands
) -> Entity {
	let word_entity = commands.spawn((
		TransformBundle {
			local : Transform::from_translation(word_description.position()),
			..default()
		},
		word_description.clone(),
		word_children.clone(),
		VisibilityBundle::default(),
		RaypickHover::default(),
	))
	.id();

	word_entity
}

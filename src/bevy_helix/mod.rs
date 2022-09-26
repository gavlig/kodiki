use bevy :: prelude :: *;

pub mod application;
pub use application :: *;

mod compositor;

mod systems;

use helix_tui 		:: { buffer :: Buffer as Surface };

pub struct BevyHelixPlugin;

impl Plugin for BevyHelixPlugin {
	fn build(&self, app: &mut App) {
        app
            .insert_resource(Surface::default())

			.add_startup_system(systems::startup.exclusive_system())
			.add_system(systems::render)
			// .add_system_to_stage(
			// 	CoreStage::PostUpdate,
			// 	on_tangent_moved
			// 		.label("bevy_spline::on_tangent_moved")
			// )
 			;
	}
}
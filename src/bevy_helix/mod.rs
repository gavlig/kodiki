use bevy :: prelude :: *;

use helix_view :: graphics :: *;

pub mod application;
pub use application :: *;
mod compositor;
mod systems;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CellBevy {
	pub dirty : bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SurfaceBevy {
	pub content : Vec<CellBevy>,
}

impl SurfaceBevy {
    /// Returns a SurfaceBevy with all cells set to the default one
    pub fn empty(area: Rect) -> SurfaceBevy {
        let cell: CellBevy = CellBevy::default();
        SurfaceBevy::filled(area, &cell)
    }

    /// Returns a SurfaceBevy with all cells initialized with the attributes of the given Cell
    pub fn filled(area: Rect, cell: &CellBevy) -> SurfaceBevy {
        let size = area.area() as usize;
        let mut content = Vec::with_capacity(size);
        for _ in 0..size {
            content.push(cell.clone());
        }
        SurfaceBevy { content }
    }
}

pub struct BevyHelixPlugin;

impl Plugin for BevyHelixPlugin {
	fn build(&self, app: &mut App) {
        app
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
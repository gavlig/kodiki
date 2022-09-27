use bevy :: prelude :: *;
use iyes_loopless :: { prelude :: * };

use crate :: game :: AppMode;

use helix_view :: graphics :: *;

pub mod spawn;
mod render;
pub mod application;
pub use application :: *;
mod compositor;
mod systems;

#[derive(Component)]
pub struct BevyHelix;

// representation of helix_tui::buffer::Cell in Bevy
#[derive(Debug, Clone, PartialEq)]
pub struct CellBevy {
    pub entity : Option<Entity>,
    pub symbol : String,
	pub dirty : bool,
}

impl Default for CellBevy {
    fn default() -> Self {
        Self {
            entity : None,
            symbol : " ".into(),
            dirty : false,
        }
    }
}

// representation of helix_tui::buffer::Buffer in Bevy
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
            .add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Main)
				.with_system(systems::render)
				.with_system(systems::input)
				.into()
			)
 			;
	}
}
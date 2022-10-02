use bevy :: prelude :: *;
use bevy :: utils :: HashMap;
use iyes_loopless :: { prelude :: * };

use crate :: game :: AppMode;

pub mod spawn;
mod render;
pub mod application;
pub use application :: *;
mod compositor;
mod editor;
mod systems;

#[derive(Component)]
pub struct BevyHelix;

#[derive(Default)]
pub struct CursorBevy {
    pub entity  : Option<Entity>,
    pub color   : Color,
    pub x       : u16,
    pub y       : u16,
    pub kind    : helix_view::graphics::CursorKind,

    pub easing_accum : f32,
}

// representation of helix_tui::buffer::Cell in Bevy
#[derive(Debug, Clone, PartialEq)]
pub struct CellBevy {
    pub entity  : Option<Entity>,
    pub symbol  : String,
    pub fg      : helix_view::graphics::Color,
    pub bg      : helix_view::graphics::Color,

    pub fg_handle : Option<Handle<StandardMaterial>>,
    pub bg_handle : Option<Handle<StandardMaterial>>,
}

impl Default for CellBevy {
    fn default() -> Self {
        Self {
            entity  : None,
            symbol  : " ".into(),
            fg      : helix_view::graphics::Color::Reset,
            bg      : helix_view::graphics::Color::Reset,

            fg_handle : None,
            bg_handle : None,
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
    pub fn empty(area: helix_view::graphics::Rect) -> SurfaceBevy {
        let cell: CellBevy = CellBevy::default();
        SurfaceBevy::filled(area, &cell)
    }

    /// Returns a SurfaceBevy with all cells initialized with the attributes of the given Cell
    pub fn filled(area: helix_view::graphics::Rect, cell: &CellBevy) -> SurfaceBevy {
        let size = area.area() as usize;
        let mut content = Vec::with_capacity(size);
        for _ in 0..size {
            content.push(cell.clone());
        }
        SurfaceBevy { content, ..default() }
    }
}

pub type MeshesMap = HashMap<String, Handle<Mesh>>;

#[derive(Default)]
pub struct TextCache {
    pub meshes: MeshesMap,
}

pub struct BevyHelixPlugin;

impl Plugin for BevyHelixPlugin {
	fn build(&self, app: &mut App) {
        app
            .insert_resource(TextCache::default())
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
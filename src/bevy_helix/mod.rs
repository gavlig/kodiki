use bevy :: prelude :: *;
use bevy :: utils :: HashMap;
use iyes_loopless :: { prelude :: * };

use helix_term :: compositor :: SurfacesMap as SurfacesMapHelix;

use crate :: game :: AppMode;

pub mod spawn;
mod render;
pub mod application;
pub use application :: *;
mod compositor;
pub mod editor;
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
    pub entity_symbol : Option<Entity>,
    pub entity_bg_quad : Option<Entity>,
    pub symbol  : String,
    pub fg      : helix_view::graphics::Color,
    pub bg      : helix_view::graphics::Color,

    pub fg_handle : Option<Handle<StandardMaterial>>,
    pub bg_handle : Option<Handle<StandardMaterial>>,
}

impl Default for CellBevy {
    fn default() -> Self {
        Self {
            entity_symbol : None,
            entity_bg_quad : None,
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
    pub entity  : Option<Entity>,
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

pub type SurfacesMapBevy = HashMap<String, SurfaceBevy>;

pub type MeshesMap = HashMap<String, Handle<Mesh>>;

pub type MaterialsMap = HashMap<String, Handle<StandardMaterial>>;

#[derive(Default)]
pub struct TextCache {
    pub meshes: MeshesMap,
}

#[derive(Default)]
pub struct HelixColorsCache {
    pub materials: MaterialsMap,
}

pub fn get_helix_color_material_handle(
	color_bevy: Color,
	helix_colors_cache: &mut MaterialsMap,
	material_assets: &mut Assets<StandardMaterial>
) -> Handle<StandardMaterial> {
    let mut color_u8 : [u8; 3] = [0; 3];
    color_u8[0] = (color_bevy.r() * 255.) as u8;
    color_u8[1] = (color_bevy.g() * 255.) as u8;
    color_u8[2] = (color_bevy.b() * 255.) as u8;
    let color_string = hex::encode(color_u8);
    match helix_colors_cache.get(&color_string) {
		Some(handle) => handle.clone_weak(),
		None => {
			let handle = material_assets.add(
				StandardMaterial {
				    base_color : color_bevy,
				    unlit : true,
				    ..default()
				}
			);

			helix_colors_cache.insert_unique_unchecked(color_string, handle).1.clone_weak()
		}
	}
}

pub struct BevyHelixPlugin;

impl Plugin for BevyHelixPlugin {
	fn build(&self, app: &mut App) {
        app
            .insert_resource(CursorBevy::default())
            .insert_resource(TextCache::default())
            .insert_resource(HelixColorsCache::default())
            .insert_resource(SurfacesMapHelix::default())

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
use bevy :: prelude :: *;
use bevy :: utils :: HashMap;
use iyes_loopless :: { prelude :: * };

use helix_term :: compositor :: SurfaceContainer as SurfaceContainerHelix;

use crate :: game :: AppMode;

mod application;
use application :: *;
mod editor;
mod compositor;

mod spawn;
mod fill;
mod render;
mod animate;
mod input;

mod systems;

#[derive(Default, Resource)]
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
	pub symbol_entity		: Option<Entity>,
	pub bg_quad_entity		: Option<Entity>,
	pub symbol				: String,
	pub fg					: helix_view::graphics::Color,
	pub bg					: helix_view::graphics::Color,

	pub fg_handle			: Option<Handle<StandardMaterial>>,
	pub bg_handle			: Option<Handle<StandardMaterial>>,
}

impl Default for CellBevy {
	fn default() -> Self {
		Self {
			symbol_entity	: None,
			bg_quad_entity	: None,
			symbol  		: " ".into(),
			fg      		: helix_view::graphics::Color::Reset,
			bg      		: helix_view::graphics::Color::Reset,

			fg_handle 		: None,
			bg_handle 		: None,
		}
	}
}

// representation of helix_tui::buffer::Buffer in Bevy
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SurfaceBevy {
	pub entity  : Option<Entity>,
	pub content : Vec<CellBevy>,
	pub area	: helix_view::graphics::Rect,
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
	
	pub fn new_with_entity(surface_entity: Entity) -> SurfaceBevy {
		SurfaceBevy { entity: Some(surface_entity), ..default() }
	}
}

pub type SurfacesMapBevyInner = HashMap<String, SurfaceBevy>;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct SurfacesMapBevy(SurfacesMapBevyInner);

pub type SurfacesMapHelixInner = HashMap<String, SurfaceContainerHelix>;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct SurfacesMapHelix(SurfacesMapHelixInner);

pub type MaterialsMap = HashMap<String, Handle<StandardMaterial>>;

#[derive(Default, Resource)]
pub struct HelixColorsCache {
	pub materials: MaterialsMap,
}

pub fn get_helix_color_material_handle(
	color_bevy			: Color,
	helix_colors_cache	: &mut HelixColorsCache,
	material_assets		: &mut Assets<StandardMaterial>
) -> Handle<StandardMaterial> {
	let mut color_u8 : [u8; 3] = [0; 3];
	color_u8[0] = (color_bevy.r() * 255.) as u8;
	color_u8[1] = (color_bevy.g() * 255.) as u8;
	color_u8[2] = (color_bevy.b() * 255.) as u8;
	let color_string = hex::encode(color_u8);
	match helix_colors_cache.materials.get(&color_string) {
		Some(handle) => handle.clone_weak(),
		None => {
			let handle = material_assets.add(
				StandardMaterial {
					base_color : color_bevy,
					unlit : true,
					..default()
				}
			);

			helix_colors_cache.materials.insert_unique_unchecked(color_string, handle).1.clone_weak()
		}
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct TokioRuntime(pub tokio::runtime::Runtime);

pub struct BevyHelixPlugin;

impl Plugin for BevyHelixPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(CursorBevy::default())
			.insert_resource(HelixColorsCache::default())
			.insert_resource(SurfacesMapHelix::default())

			.insert_resource(TokioRuntime{ 0: tokio::runtime::Builder::new_multi_thread()
				.enable_all()
				.build()
				.unwrap()
			})

			.add_exit_system_set(AppMode::AssetsLoaded,
				ConditionSet::new()
				.run_in_state(AppMode::AssetsLoaded)
				.with_system(systems::startup_app)
				.with_system(systems::startup_spawn)
				.into()
			)
			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Main)
				.with_system(systems::tick)
				.with_system(systems::tokio_events)
				.with_system(systems::input_keyboard)
				.into()
			)
			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Reader)
				.with_system(systems::tick)
				.with_system(systems::tokio_events)
				.into()
			)
			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Fly)
				.with_system(systems::tick)
				.with_system(systems::tokio_events)
				.into()
			)
 			;
	}
}
use bevy :: prelude :: *;
use bevy :: utils :: HashMap;
use iyes_loopless :: { prelude :: * };

use helix_term :: compositor :: SurfaceContainer as SurfaceContainerHelix;

use crate :: { game :: AppMode, bevy_ab_glyph :: StringWithFonts };

mod application;
use application :: *;
mod spawn;
mod update;
mod animate;
mod input;

mod systems;

#[derive(Default, Resource)]
pub struct CursorBevy {
	pub entity  	: Option<Entity>,
	pub color   	: Color,
	pub x       	: u32,
	pub y       	: u32,
	pub kind    	: helix_view::graphics::CursorKind,

	pub easing_accum : f32,
}

#[derive(Component, Clone, Debug)]
pub struct WordDescription {
	pub string	: String,
	pub row		: u32,
	pub column	: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordBevy {
	pub entity		: Option<Entity>,
	pub string		: String,
	pub color		: helix_view::graphics::Color,
	pub column		: u32,
}

pub type WordRowBevy	= Vec<WordBevy>;
pub type WordRowsBevy	= Vec<WordRowBevy>;

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundQuadBevy {
	pub entity		: Option<Entity>,
	pub color		: helix_view::graphics::Color,
	pub column		: u32,
	pub length		: u32,
}

pub type BackgroundQuadRowBevy = Vec<BackgroundQuadBevy>;
pub type BackgroundQuadRowsBevy	= Vec<BackgroundQuadRowBevy>;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct RowBevy {
	pub words		: WordRowBevy,
	pub quads		: BackgroundQuadRowBevy,
}

impl RowBevy {
	pub fn clear(&mut self) {
		self.words.clear();
		self.quads.clear();
	}
}

pub type RowsBevy = Vec<RowBevy>;

// representation of helix_tui::buffer::Buffer in Bevy
#[derive(Clone, PartialEq, Debug)]
pub struct SurfaceBevy {
	pub entity  			: Option<Entity>,
	pub background_entity	: Option<Entity>,
	pub rows				: RowsBevy,
	pub row_offset_global	: i32,
	pub row_offset_local	: i32,
	pub area				: helix_view::graphics::Rect,
	
	pub update				: bool,
}

impl Default for SurfaceBevy {
	fn default() -> Self {
		Self {
			entity				: None,
			background_entity	: None,
			rows				: RowsBevy::new(),
			row_offset_global	: 0,
			row_offset_local	: 0,
			area				: helix_view::graphics::Rect::default(),
			update				: true,
		}
	}
}

impl SurfaceBevy {
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
				.with_system(systems::update_main)
				.with_system(systems::tokio_events)
				.with_system(systems::input_keyboard)
				// .with_system(systems::despawn_culled_words)
				.into()
			)
			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Reader)
				.with_system(systems::update_main)
				.with_system(systems::tokio_events)
				// .with_system(systems::despawn_culled_words)
				.into()
			)
			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Fly)
				.with_system(systems::update_main)
				.with_system(systems::tokio_events)
				// .with_system(systems::despawn_culled_words)
				.into()
			)
 			;
	}
}
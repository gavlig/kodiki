use bevy :: prelude :: { * };
use bevy :: utils :: { HashMap };
use iyes_loopless :: prelude :: { * };
use bevy_contrib_colors	:: { Tailwind };

use helix_view::graphics::Color as HelixColor;

use crate :: game :: AppMode;

mod application;
use application :: *;
mod surface;
use surface :: *;
mod cursor;
use cursor :: *;
mod spawn;
mod animate;
mod input;
mod utils;

mod systems;

fn color_from_helix(helix_color: HelixColor) -> Color {
	match helix_color {
		HelixColor::Reset		=> Color::WHITE,
		HelixColor::Black		=> Color::BLACK,
		HelixColor::Red			=> Tailwind::RED600,
		HelixColor::Green		=> Tailwind::GREEN600,
		HelixColor::Yellow		=> Tailwind::YELLOW600,
		HelixColor::Blue		=> Tailwind::BLUE600,
		HelixColor::Magenta		=> Tailwind::PURPLE600,
		HelixColor::Cyan		=> Color::rgb(0.0, 0.5, 0.5),
		HelixColor::Gray		=> Tailwind::GRAY600,
		HelixColor::LightRed	=> Tailwind::RED300,
		HelixColor::LightGreen	=> Tailwind::GREEN300,
		HelixColor::LightBlue	=> Tailwind::BLUE300,
		HelixColor::LightYellow => Tailwind::YELLOW300,
		HelixColor::LightMagenta => Tailwind::PURPLE300,
		HelixColor::LightCyan	=> Color::rgb(0.0, 0.7, 0.7),
		HelixColor::LightGray	=> Tailwind::GRAY300,
		HelixColor::White		=> Color::WHITE,
		// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
		HelixColor::Indexed(_i) => { panic!("Indexed color is not supported!"); }, // Color::AnsiValue(i), 
		HelixColor::Rgb(r, g, b) => Color::rgb_u8(r, g, b),
	}
}

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
				.with_system(systems::update_editor_background_quad)
				.into()
			)
			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Reader)
				.with_system(systems::update_main)
				.with_system(systems::tokio_events)
				.with_system(systems::update_editor_background_quad)
				.into()
			)
			.add_system_set(
				ConditionSet::new()
				.run_in_state(AppMode::Fly)
				.with_system(systems::update_main)
				.with_system(systems::tokio_events)
				.into()
			)
 			;
	}
}
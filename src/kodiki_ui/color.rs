use bevy :: prelude :: *;

use super :: *;

pub const EMISSIVE_MULTIPLIER_STRONG : f32 = 60.;
pub const EMISSIVE_MULTIPLIER_MEDIUM : f32 = 40.;
pub const EMISSIVE_MULTIPLIER_SMALL : f32 = 30.;

pub fn get_color_material_handle(
	color				: Color,
	color_materials_cache : &mut ColorMaterialsCache,
	material_assets		: &mut Assets<StandardMaterial>
) -> Handle<StandardMaterial> {
	get_color_material_walpha_handle(color, AlphaMode::Opaque, color_materials_cache, material_assets)
}

pub fn get_emissive_material_handle(
	color				: Color,
	color_materials_cache : &mut ColorMaterialsCache,
	material_assets		: &mut Assets<StandardMaterial>
) -> Handle<StandardMaterial> {
	let unlit = false;
	get_material_walpha_handle(Color::BLACK, color, AlphaMode::Opaque, unlit, color_materials_cache, material_assets)
}

pub fn get_color_material_walpha_handle(
	color				: Color,
	alpha_mode			: AlphaMode,
	color_materials_cache : &mut ColorMaterialsCache,
	material_assets		: &mut Assets<StandardMaterial>
) -> Handle<StandardMaterial> {
	let unlit = true;
	get_material_walpha_handle(color, Color::BLACK, alpha_mode, unlit, color_materials_cache, material_assets)
}

pub fn get_material_walpha_handle(
	color				: Color,
	emissive			: Color,
	alpha_mode			: AlphaMode,
	unlit				: bool,
	color_materials_cache : &mut ColorMaterialsCache,
	material_assets		: &mut Assets<StandardMaterial>
) -> Handle<StandardMaterial> {
	let mut color_u8 : [u8; 9] = [0; 9];

	color_u8[0] = (color.r() * 255.) as u8;
	color_u8[1] = (color.g() * 255.) as u8;
	color_u8[2] = (color.b() * 255.) as u8;
	color_u8[3] = (color.a() * 255.) as u8;

	color_u8[4] = (emissive.r() / EMISSIVE_MULTIPLIER_STRONG * 255.) as u8;
	color_u8[5] = (emissive.g() / EMISSIVE_MULTIPLIER_STRONG * 255.) as u8;
	color_u8[6] = (emissive.b() / EMISSIVE_MULTIPLIER_STRONG * 255.) as u8;

	color_u8[7] = unlit as u8;
	color_u8[8] = (alpha_mode == AlphaMode::Opaque) as u8;

	let color_string = hex::encode(color_u8);

	match color_materials_cache.materials.get(&color_string) {
		Some(handle) => handle.clone_weak(),
		None => {
			let handle = material_assets.add(
				StandardMaterial {
					base_color : color,
					emissive,
					alpha_mode,
					unlit,
					..default()
				}
			);

			color_materials_cache.materials.insert_unique_unchecked(color_string, handle).1.clone_weak()
		}
	}
}

pub fn get_color_as_modified_hsla(color: Color, hue_add: f32, saturation_add: f32, lightness_add: f32, alpha_add: f32) -> Color {
	match color.as_hsla() {
		Color::Hsla { hue, saturation, lightness, alpha } => {
			Color::Hsla {
				hue			: (hue			+ hue_add)			.min(360.0),
				saturation	: (saturation	+ saturation_add)	.min(1.0),
				lightness	: (lightness	+ lightness_add)	.min(1.0),
				alpha		: (alpha		+ alpha_add)		.min(1.0)
			}
		},
		_ => { debug_assert!(false); Color::CYAN } // this shouldn't happen ever
	}
}

pub fn get_color_wmodified_lightness(color: Color, lightness_add: f32) -> Color {
	get_color_as_modified_hsla(color, 0.0, 0.0, lightness_add, 0.0)
}
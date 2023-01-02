use bevy :: prelude :: { * };
use bevy :: render :: texture :: { ImageType };

use ab_glyph :: { Font, FontVec };

use super :: { EmojiMaterialsCache, GlyphWithFonts };

fn generate_glyph_image_char(
	glyph_char	: char,
	font		: &FontVec,
	debug		: bool,
) -> Image {
	let glyph_id = font.glyph_id(glyph_char);

	if debug {
		println!("generate_glyph_image glyph_id: {:?} char {}", glyph_id, glyph_char);
	}

	if let Some(glyph_image) = font.glyph_raster_image(glyph_id, u16::MAX) {
		let ext = match glyph_image.format {
			ab_glyph::GlyphImageFormat::Png => "png",
			_ => panic!("unsupported glyph image format obtained from ab_glyph!")
		};
		
		let compressed_image_formats = bevy::render::texture::CompressedImageFormats::NONE;
		
		let buffer_result = Image::from_buffer(
		    glyph_image.data,
		    ImageType::Extension(ext),
		    compressed_image_formats,
		    true,
		);
		
		if let Ok(buffer) = buffer_result {
			buffer
		} else {
			error!("Failed to create emoji image for glyph: {}", glyph_char);
			Image::default()
		}
	} else {
		error!("Failed to obtain emoji image from ab_glyph! glyph: {}", glyph_char);
		Image::default()
	}
}

fn generate_glyph_image(
	glyph_str	: &String,
	font		: &FontVec,
	debug		: bool,
) -> Image {
	generate_glyph_image_char(
		glyph_str.chars().next().unwrap(),
		font,
		debug
	)
}

pub fn generate_emoji_material_wcache(
	glyph_with_fonts	: &GlyphWithFonts,
	image_assets		: &mut Assets<Image>,
	material_assets		: &mut Assets<StandardMaterial>,
	emoji_materials_cache : &mut EmojiMaterialsCache
) -> Handle<StandardMaterial> {
	match emoji_materials_cache.materials.get(&glyph_with_fonts.glyph_str) {
		Some(handle) => handle.clone_weak(),
		None => {
			let image_handle = image_assets.add(
				generate_glyph_image(&glyph_with_fonts.glyph_str, &glyph_with_fonts.current_font().f, false)
			);
			
			let material_handle = material_assets.add(StandardMaterial{
				base_color_texture: Some(image_handle),
				unlit: true,
				..default()
			});
			
			emoji_materials_cache.materials.insert_unique_unchecked(glyph_with_fonts.glyph_str.clone(), material_handle).1.clone()
		}
	}
}
use bevy :: {
	prelude :: *,
	render :: {
		texture :: ImageType,
		mesh :: Mesh,
	}
};

use ab_glyph :: { Font, FontVec };

use super :: { EmojiMaterialsCache, TextMeshesCache, GlyphWithFonts };
use super :: generator_common :: *;

fn generate_emoji_image_char(
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

		let compressed_image_formats = bevy::render::texture::CompressedImageFormats::all();

		let image_result = Image::from_buffer(
		    glyph_image.data,
		    ImageType::Extension(ext),
		    compressed_image_formats,
		    true,
		);

		if let Ok(image) = image_result {
			image
		} else {
			error!("Failed to create emoji image for glyph: {}", glyph_char);
			Image::default()
		}
	} else {
		error!("Failed to obtain emoji image from ab_glyph! glyph: {}", glyph_char);
		Image::default()
	}
}

fn generate_emoji_image(
	glyph_str	: &String,
	font		: &FontVec,
	debug		: bool,
) -> Image {
	generate_emoji_image_char(
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
	match emoji_materials_cache.materials.get(glyph_with_fonts.glyph_str) {
		Some(handle) => handle.clone_weak(),
		None => {
			let image_handle = image_assets.add(
				generate_emoji_image(&glyph_with_fonts.glyph_str, &glyph_with_fonts.current_font().f, false)
			);

			let material_handle = material_assets.add(StandardMaterial{
				base_color_texture: Some(image_handle),
				alpha_mode: AlphaMode::Blend,
				unlit: true,
				..default()
			});

			emoji_materials_cache.materials.insert_unique_unchecked(glyph_with_fonts.glyph_str.clone(), material_handle).1.clone()
		}
	}
}

pub fn generate_emoji_mesh_internal(
	glyph_with_fonts : &GlyphWithFonts,
) -> MeshInternal {
	let glyph_str			= glyph_with_fonts.glyph_str;
	let font				= glyph_with_fonts.current_font();

	let mut mesh 			= MeshInternal::default();
	generate_quad_vertices	(&mut mesh, font.scale);

	let glyph_id = font.glyph_id(glyph_str);

	if let Some(glyph_image) = font.f.glyph_raster_image(glyph_id, u16::MAX) {
		for v in mesh.vertex_buffer.vertices.iter_mut() {
			v[1] += (glyph_image.origin.y / glyph_image.scale) * font.scale - font.descent();
		}
	} else {
		error!("Failed to obtain emoji image from ab_glyph! glyph: {}", glyph_str);
	}

	mesh
}

pub fn generate_emoji_mesh_wcache(
	glyph_with_fonts	: &GlyphWithFonts,
	mesh_assets			: &mut Assets<Mesh>,
	text_meshes_cache 	: &mut TextMeshesCache
) -> Handle<Mesh> {
	match text_meshes_cache.meshes.get(glyph_with_fonts.glyph_str) {
		Some(handle) => handle.clone_weak(),
		None => {
			let mesh_internal = generate_emoji_mesh_internal(glyph_with_fonts);

			let handle = mesh_assets.add(
				bevy_mesh_from_internal(&mesh_internal)
			);

			text_meshes_cache.meshes.insert_unique_unchecked(glyph_with_fonts.glyph_str.clone(), handle).1.clone()
		}
	}
}
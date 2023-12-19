use bevy :: prelude :: *;
use bevy :: reflect :: TypeUuid;
use bevy :: utils	:: HashMap;

use ab_glyph :: { Font, FontVec, GlyphId };

use std :: {
	path :: PathBuf,
	sync :: Arc,
};

mod generator_common;
pub mod font_loader;
pub mod glyph_image_generator;
pub mod glyph_mesh_generator;
pub mod emoji_generator;

pub use font_loader :: FontLoader;

use glyph_mesh_generator :: generate_string_mesh_wcache;
use emoji_generator :: generate_emoji_mesh_wcache;

#[derive(Asset, TypePath, TypeUuid, Debug)]
#[uuid = "1a92e0e6-6915-11ed-9022-0242ac120002"]
pub struct ABGlyphFont {
	pub f			: Arc<FontVec>,
	pub path		: PathBuf,

	pub scale		: f32,
	pub thickness	: f32, // how thick the mesh is.
	pub tolerance	: f32, // how detailed the mesh is. bigger number means less details
}

impl PartialEq for ABGlyphFont {
	fn eq(&self, other: &Self) -> bool {
		self.path == other.path && self.scale == other.scale && self.thickness == other.thickness && self.tolerance == other.tolerance
	}
}

impl ABGlyphFont {
	pub fn glyph_id(&self, glyph_str: &String) -> GlyphId {
		self.f.glyph_id(glyph_str.chars().next().unwrap())
	}

	pub fn glyph_id_char(&self, glyph_char: char) -> GlyphId {
		self.f.glyph_id(glyph_char)
	}

	pub fn vertical_advance(&self) -> f32 {
		let unit_scale = self.f.units_per_em().unwrap();

		let advance_unscaled = (self.f.height_unscaled() + self.f.line_gap_unscaled()) / unit_scale;
		let advance = advance_unscaled * self.scale;

		advance
	}

	pub fn horizontal_advance(&self, glyph_str: &String) -> f32 {
		let glyph_char = glyph_str.chars().next().unwrap();
		self.horizontal_advance_char(glyph_char)
	}

	pub fn horizontal_advance_char(&self, glyph_char: char) -> f32 {
		let unit_scale = self.f.units_per_em().unwrap();

		let glyph_id = self.glyph_id_char(glyph_char);
		let advance_unscaled = self.f.h_advance_unscaled(glyph_id) / unit_scale;
		let advance = advance_unscaled * self.scale;

		advance
	}

	pub fn horizontal_advance_mono(&self) -> f32 {
		self.horizontal_advance_char('a')
	}

	pub fn horizontal_advance_mono_unscaled(&self) -> f32 {
		self.horizontal_advance_mono() / self.scale
	}

	pub fn horizontal_advance_mono_rescaled(&self, rescale: f32) -> f32 {
		(self.horizontal_advance_mono() / self.scale) * rescale
	}

	pub fn kerning(&self, glyph_str0: &String, glyph_str1: &String) -> f32 {
		let unit_scale = self.f.units_per_em().unwrap();

		let glyph_id0 = self.glyph_id(glyph_str0);
		let glyph_id1 = self.glyph_id(glyph_str1);

		let kern_unscale = self.f.kern_unscaled(glyph_id0, glyph_id1) / unit_scale;
		let kern = kern_unscale * self.scale;

		kern
	}

	pub fn descent(&self) -> f32 {
		let unit_scale = self.f.units_per_em().unwrap();
		(self.f.descent_unscaled() / unit_scale) * self.scale
	}

	pub fn is_emoji(&self) -> bool {
		GlyphId(0) != self.glyph_id_char('✅')
	}
}

unsafe impl Sync for ABGlyphFont {}
unsafe impl Send for ABGlyphFont {}

#[derive(Resource, Default, Debug)]
pub struct FontAssetHandles {
	pub main			: Handle<ABGlyphFont>,
	pub emoji			: Handle<ABGlyphFont>,
	pub fallback		: Vec<Handle<ABGlyphFont>>,

	pub loaded_cnt		: usize,
}

impl FontAssetHandles {
	pub fn handles_total(&self) -> usize {
		self.fallback.len() + 2
	}

	pub fn loaded(&self) -> bool {
		self.loaded_cnt == self.handles_total()
	}
}

#[derive(Debug)]
pub struct ABGlyphFonts<'a> {
	pub main			: &'a ABGlyphFont,
	pub emoji			: &'a ABGlyphFont,
	pub fallback		: Vec<&'a ABGlyphFont>,
}

impl ABGlyphFonts<'_> {
	pub fn new<'a>(
		font_assets		: &'a Assets<ABGlyphFont>,
		font_handles	: &'a FontAssetHandles,
	) -> ABGlyphFonts<'a>
	{
		let main 	= font_assets.get(&font_handles.main).unwrap();
		let emoji	= font_assets.get(&font_handles.emoji).unwrap();

		let mut fallback = Vec::new();
		for handle in font_handles.fallback.iter() {
			fallback.push(font_assets.get(handle.id()).unwrap());
		}

		ABGlyphFonts {
			main,
			emoji,
			fallback
		}
	}

	pub fn is_emoji(&self, string: &String) -> bool {
		let mut result = true;

		for current_char in string.chars() {
			let string_from_char = String::from(current_char);
			let symbol = GlyphWithFonts::new(&string_from_char, self);

			result &= symbol.is_emoji;
		}

		result
	}

	pub fn generate_string_mesh(
		&self,
		string					: &String,
		glyph_meshes_cache		: &mut GlyphMeshesCache,
		text_meshes_cache		: &mut TextMeshesCache,
		mesh_assets				: &mut Assets<Mesh>,
	) -> Handle<Mesh> {
		let first_char = string.chars().next().unwrap();
		let first_char_string = String::from(first_char);
		let first_symbol = GlyphWithFonts::new(&first_char_string, self);

		// normal glyphs are made of meshes with simple color-material, emojis are made of simple quad mesh with image-material
		if first_symbol.is_emoji {
			generate_emoji_mesh_wcache(&first_symbol, mesh_assets, text_meshes_cache)
		} else {
			generate_string_mesh_wcache(string, first_symbol.current_font(), mesh_assets, glyph_meshes_cache, text_meshes_cache)
		}
	}
}

#[derive(Clone, Debug)]
pub struct GlyphWithFonts<'a> {
	pub glyph_str	: &'a String,
	pub fonts		: &'a ABGlyphFonts<'a>,

	pub initialized	: bool,
	pub is_emoji	: bool,
	pub fallback_index	: Option<usize>,
}

impl GlyphWithFonts<'_> {
	pub fn new<'a>(
		glyph_str	: &'a String,
		fonts		: &'a ABGlyphFonts
	) -> GlyphWithFonts<'a>
	{
		let mut cwf = GlyphWithFonts {
			glyph_str,
			fonts,

			initialized		: false,
			is_emoji		: false,
			fallback_index	: None,
		};

		cwf.initialize();

		cwf
	}

	fn is_main_font(&self) -> bool {
		GlyphId(0) != self.fonts.main.glyph_id(self.glyph_str)
	}

	fn is_emoji(&self) -> bool {
		GlyphId(0) != self.fonts.emoji.glyph_id(self.glyph_str)
	}

	fn find_fallback_index(&self) -> Option<usize>
	{
		let mut glyph_id		= self.fonts.main.glyph_id(self.glyph_str);
		let mut fallback_index	= None;
		if glyph_id == GlyphId(0) {
			for (index, fallback_font) in self.fonts.fallback.iter().enumerate() {
				glyph_id = fallback_font.glyph_id(self.glyph_str);
				if glyph_id != GlyphId(0) {
					fallback_index = Some(index);
					break;
				}
			}
		}

		fallback_index
	}

	pub fn initialize(&mut self) {
		// check glyph availablility in order: main font -> fallback font -> emoji font

		let main			= self.is_main_font();
		if !main {
			self.fallback_index = self.find_fallback_index();

			if None == self.fallback_index {
				self.is_emoji = self.is_emoji();
			}
		}

		self.initialized	= true;
	}

	pub fn current_font(&self) -> &ABGlyphFont {
		assert!(self.initialized);

		if self.is_emoji {
			self.fonts.emoji
		} else if self.fallback_index.is_some() {
			self.fonts.fallback[self.fallback_index.unwrap()]
		} else {
			self.fonts.main
		}
	}
}

pub type GlyphMeshesMap = HashMap<char, generator_common::MeshInternal>;

#[derive(Resource, Default)]
pub struct GlyphMeshesCache {
	pub meshes: GlyphMeshesMap,
}

pub type TextMeshesMap = HashMap<String, Handle<Mesh>>;

#[derive(Resource, Default)]
pub struct TextMeshesCache {
	pub meshes: TextMeshesMap,
}

pub type EmojiMaterialsMap = HashMap<String, Handle<StandardMaterial>>;

#[derive(Resource, Default)]
pub struct EmojiMaterialsCache {
	pub materials: EmojiMaterialsMap,
}


pub struct ABGlyphPlugin;

impl Plugin for ABGlyphPlugin {
	fn build(&self, app: &mut App) {
		app
			.register_asset_loader(FontLoader)
		    .init_asset::<ABGlyphFont>()

			.insert_resource	(GlyphMeshesCache::default())
			.insert_resource	(TextMeshesCache::default())
			.insert_resource	(FontAssetHandles::default())
			.insert_resource	(EmojiMaterialsCache::default())

//			.add_system         (mesh_generator::ab_glyph_curve_debug_system)
			;
	}
}
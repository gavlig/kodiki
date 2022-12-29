use bevy :: prelude :: { * };
use bevy :: reflect :: TypeUuid;
use bevy :: utils	:: HashMap;

use ab_glyph :: { Font, FontVec, GlyphId };

use std :: path::{ PathBuf };

#[derive(TypeUuid, Debug)]
#[uuid = "1a92e0e6-6915-11ed-9022-0242ac120002"]
pub struct ABGlyphFont {
	pub f			: FontVec,
	pub path		: PathBuf,

	pub scale		: f32,
	pub depth		: f32, // how thick the mesh is.
	pub tolerance	: f32, // how detailed the mesh is. bigger number means less details
}

impl PartialEq for ABGlyphFont {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.scale == other.scale && self.depth == other.depth && self.tolerance == other.tolerance
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

	pub fn kerning(&self, glyph_str0: &String, glyph_str1: &String) -> f32 {
		let unit_scale = self.f.units_per_em().unwrap();

		let glyph_id0 = self.glyph_id(glyph_str0);
		let glyph_id1 = self.glyph_id(glyph_str1);

		let kern_unscale = self.f.kern_unscaled(glyph_id0, glyph_id1) / unit_scale;
		let kern = kern_unscale * self.scale;

		kern
	}

	pub fn depth_scaled(&self) -> f32 {
		self.depth * self.scale
	}

	pub fn vertical_down_offset(&self) -> f32 {
		self.vertical_advance() / 4.5
	}
}

unsafe impl Sync for ABGlyphFont {}
unsafe impl Send for ABGlyphFont {}

pub struct UsedFonts<'a> {
	pub main		: &'a ABGlyphFont,
	pub fallback	: &'a ABGlyphFont,
}

#[derive(Clone, Debug)]
pub struct GlyphWithFonts<'a> {
	pub glyph_str	: String,
	pub main		: &'a ABGlyphFont,
	pub fallback	: &'a ABGlyphFont,

	pub initialized	: bool,
	pub use_fallback: bool,
}

impl GlyphWithFonts<'_> {
	pub fn new<'a>(
		glyph_str	: String,
		used_fonts	: &'a UsedFonts
	) -> GlyphWithFonts<'a>
	{
		let mut cwf = GlyphWithFonts {
			glyph_str	: glyph_str,
			main		: used_fonts.main,
			fallback	: used_fonts.fallback,
		
			initialized	: false,
			use_fallback: false,
		};

		cwf.initialize();

		cwf
	}

	pub fn use_fallback_font(
		char_with_fonts	: &GlyphWithFonts,
	) -> bool
	{
		let glyph_id = char_with_fonts.main.glyph_id(&char_with_fonts.glyph_str);
		let use_fallback = glyph_id == GlyphId(0);
		if use_fallback {
			let glyph_id = char_with_fonts.fallback.glyph_id(&char_with_fonts.glyph_str);
			assert!(glyph_id != GlyphId(0), "couldnt find glyph for {:?}!", char_with_fonts.glyph_str);
		}
	
		use_fallback
	}

	pub fn initialize(&mut self) {
		self.use_fallback = GlyphWithFonts::use_fallback_font(&self);
		self.initialized = true;
	}

	pub fn current_font(&self) -> &ABGlyphFont {
		assert!(self.initialized);

		if self.use_fallback {
			self.fallback
		} else {
			self.main
		}
	}
}

pub type StringWithFonts<'a> = Vec<GlyphWithFonts<'a>>;

pub type GlyphMeshesMap = HashMap<String, mesh_generator::MeshInternal>;

#[derive(Resource, Default)]
pub struct GlyphMeshesCache {
	pub meshes: GlyphMeshesMap,
}

pub type TextMeshesMap = HashMap<String, Handle<Mesh>>;

#[derive(Resource, Default)]
pub struct TextMeshesCache {
	pub meshes: TextMeshesMap,
}

mod font_loader;
pub mod mesh_generator;

pub struct ABGlyphPlugin;

impl Plugin for ABGlyphPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource	(GlyphMeshesCache::default())
			.insert_resource	(TextMeshesCache::default())
			.add_asset          :: <ABGlyphFont>()
			.init_asset_loader  :: <font_loader::FontLoader>()

//			.add_system         (mesh_generator::ab_glyph_curve_debug_system)
			;
	}
}
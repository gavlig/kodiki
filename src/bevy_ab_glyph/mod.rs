use bevy :: prelude :: { * };
use bevy :: reflect :: TypeUuid;
use bevy :: utils	:: HashMap;

use ab_glyph :: { Font, FontVec, GlyphId };

#[derive(TypeUuid)]
#[uuid = "1a92e0e6-6915-11ed-9022-0242ac120002"]
pub struct ABGlyphFont {
	pub f			: FontVec,

	pub scale		: f32,
	pub depth		: f32, // how thick the mesh is.
	pub tolerance	: f32, // how detailed the mesh is. bigger number means less details
}

impl ABGlyphFont {
	pub fn glyph_id(&self, glyph_str: &String) -> GlyphId {
		self.f.glyph_id(glyph_str.chars().next().unwrap())
	}

	pub fn vertical_advance(&self) -> f32 {
		let unit_scale = self.f.units_per_em().unwrap();

		let advance_unscaled = (self.f.height_unscaled() + self.f.line_gap_unscaled()) / unit_scale;
		let advance = advance_unscaled * self.scale;

		advance
	}

	pub fn horizontal_advance(&self, glyph_str: &String) -> f32 {
		let unit_scale = self.f.units_per_em().unwrap();

		let glyph_id = self.glyph_id(glyph_str);
		let advance_unscaled = self.f.h_advance_unscaled(glyph_id) / unit_scale;
		let advance = advance_unscaled * self.scale;

		advance
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

pub struct UsedFonts<'a> {
	pub main		: &'a ABGlyphFont,
	pub fallback	: &'a ABGlyphFont,
}

pub struct CharWithFonts<'a> {
	pub glyph_str	: String,
	pub main		: &'a ABGlyphFont,
	pub fallback	: &'a ABGlyphFont,

	pub initialized	: bool,
	pub use_fallback: bool,
}

impl CharWithFonts<'_> {
	pub fn new<'a>(
		glyph_str	: String,
		used_fonts	: &'a UsedFonts
	) -> CharWithFonts<'a>
	{
		let mut cwf = CharWithFonts {
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
		char_with_fonts	: &CharWithFonts,
	) -> bool
	{
		let glyph_id = char_with_fonts.main.glyph_id(&char_with_fonts.glyph_str);
		let use_fallback = glyph_id == GlyphId(0);
		if use_fallback {
			let glyph_id = char_with_fonts.fallback.glyph_id(&char_with_fonts.glyph_str);
			assert!(glyph_id != GlyphId(0));
		}
	
		use_fallback
	}

	pub fn initialize(&mut self) {
		self.use_fallback = CharWithFonts::use_fallback_font(&self);
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

unsafe impl Sync for ABGlyphFont {}
unsafe impl Send for ABGlyphFont {}

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
			.insert_resource	(TextMeshesCache::default())
			.add_asset          :: <ABGlyphFont>()
			.init_asset_loader  :: <font_loader::FontLoader>()

//			.add_system         (mesh_generator::ab_glyph_curve_debug_system)
			;
	}
}
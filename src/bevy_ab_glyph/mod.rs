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

#[derive(Resource, Default, Debug)]
pub struct FontAssetHandles {
	pub main			: Handle<ABGlyphFont>,
	pub emoji			: Handle<ABGlyphFont>,
	pub fallback		: Vec<Handle<ABGlyphFont>>,

	pub loaded_cnt		: usize,
}

#[derive(Debug)]
pub struct ABFonts<'a> {
	pub main			: &'a ABGlyphFont,
	pub emoji			: &'a ABGlyphFont,
	pub fallback		: Vec<&'a ABGlyphFont>,
}

impl ABFonts<'_> {
	pub fn new<'a>(
		font_assets		: &'a Res<'a, Assets<ABGlyphFont>>,
		font_handles	: &'a Res<'a, FontAssetHandles>,
	) -> ABFonts<'a>
	{
		let main 	= font_assets.get(&font_handles.main).unwrap();
		let emoji	= font_assets.get(&font_handles.emoji).unwrap();
		
		let mut fallback = Vec::new();
		for handle in font_handles.fallback.iter() {
			fallback.push(font_assets.get(&handle).unwrap());
		}
		
		ABFonts {
			main,
			emoji,
			fallback
		}
	}
}

#[derive(Clone, Debug)]
pub struct GlyphWithFonts<'a> {
	pub glyph_str	: String,
	pub fonts		: &'a ABFonts<'a>,

	pub initialized	: bool,
	pub is_emoji	: bool,
	pub fallback_index	: Option<usize>,
}

impl GlyphWithFonts<'_> {
	pub fn new<'a>(
		glyph_str	: String,
		fonts		: &'a ABFonts
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
	
	fn main(&self) -> bool {
		GlyphId(0) != self.fonts.main.glyph_id(&self.glyph_str)
	}
	
	fn is_emoji(&self) -> bool {
		GlyphId(0) != self.fonts.emoji.glyph_id(&self.glyph_str)
	}

	fn find_fallback_index(&self) -> Option<usize>
	{
		let mut glyph_id		= self.fonts.main.glyph_id(&self.glyph_str);
		let mut fallback_index	= None;
		if glyph_id == GlyphId(0) {
			for (index, fallback_font) in self.fonts.fallback.iter().enumerate() {
				glyph_id = fallback_font.glyph_id(&self.glyph_str);
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
		
		let main			= self.main();
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

pub type StringWithFonts<'a> = Vec<GlyphWithFonts<'a>>;

pub type GlyphMeshesMap = HashMap<String, generator_common::MeshInternal>;

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

mod font_loader;
mod generator_common;
pub mod glyph_generator;
pub mod emoji_generator;

pub struct ABGlyphPlugin;

impl Plugin for ABGlyphPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource	(GlyphMeshesCache::default())
			.insert_resource	(TextMeshesCache::default())
			.insert_resource	(FontAssetHandles::default())
			.insert_resource	(EmojiMaterialsCache::default())
			.add_asset          :: <ABGlyphFont>()
			.init_asset_loader  :: <font_loader::FontLoader>()

//			.add_system         (mesh_generator::ab_glyph_curve_debug_system)
			;
	}
}
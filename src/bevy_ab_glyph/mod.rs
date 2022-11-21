use bevy :: prelude :: { * };
use bevy :: reflect :: TypeUuid;

use ab_glyph :: FontVec;

#[derive(TypeUuid)]
#[uuid = "1a92e0e6-6915-11ed-9022-0242ac120002"]
pub struct ABGlyphFont {
    pub f: ab_glyph::FontVec,
}

unsafe impl Sync for ABGlyphFont {} // FIXME - verify the soundness
unsafe impl Send for ABGlyphFont {} // FIXME - verify the soundness

mod font_loader;
pub mod mesh_generator;

pub struct ABGlyphPlugin;

impl Plugin for ABGlyphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset          :: <ABGlyphFont>()
            .init_asset_loader  :: <font_loader::FontLoader>();
    }
}
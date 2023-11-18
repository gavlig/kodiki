use std :: path :: PathBuf;
use std :: sync :: Arc;

use bevy :: asset :: { AssetLoader, BoxedFuture, LoadContext, LoadedAsset };

use ab_glyph :: FontVec;

use super :: ABGlyphFont;

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
	fn load<'a>(
		&'a self,
		bytes: &'a [u8],
		load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, anyhow::Result<()>> {
		Box::pin(async move {
			let f = FontVec::try_from_vec(bytes.to_vec())?;
			let font = ABGlyphFont {
				f:			Arc::new(f),
				path:		PathBuf::from(load_context.path()),
				scale:      0.1,
				thickness:	0.01,
				tolerance:  1.0,
			};

			load_context.set_default_asset(LoadedAsset::new(font));

			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["otf", "ttf"]
	}
}

use std :: path :: PathBuf;
use std :: sync :: Arc;

use bevy :: asset :: { AssetLoader, BoxedFuture, LoadContext, io::Reader };

use futures_lite :: AsyncReadExt;

use ab_glyph :: FontVec;

use super :: ABGlyphFont;

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
	type Asset = ABGlyphFont;
    type Settings = ();
	type Error = anyhow::Error;

	fn load<'a>(
		&'a self,
		reader: &'a mut Reader,
		_settings: &'a Self::Settings,
		load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
		Box::pin(async move {
			let mut bytes = Vec::new();
			reader.read_to_end(&mut bytes).await?;

			let f = FontVec::try_from_vec(bytes.to_vec())?;
			let font = ABGlyphFont {
				f:			Arc::new(f),
				path:		PathBuf::from(load_context.path()),
				scale:      0.1,
				thickness:	0.01,
				tolerance:  1.0,
			};

			Ok(font)
		})
	}

	fn extensions(&self) -> &[&str] {
		&["otf", "ttf"]
	}
}

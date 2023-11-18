use bevy		:: { prelude :: *, render :: { texture :: TextureFormatPixelInfo, render_resource::TextureFormat } };

use ab_glyph	:: { Font, FontVec, PxScale, PxScaleFont, ScaleFont, OutlinedGlyph };

use std :: sync :: Arc;

pub fn clear_image_range(
	font		: &Arc<FontVec>,
	scale		: f32,
	color		: Color,
	mut from	: TablePos,
	mut to		: TablePos,
	image_out	: &mut Image,
) {
	debug_assert!(image_out.texture_descriptor.format == TextureFormat::Bgra8Unorm);

	// keeping row as it is will render row 0 above visible area because bounds.min.y is negative
	from.row		+= 1;
	to.row			+= 1;

	let scale		= PxScale::from(scale);
    let scaled_font	= font.as_scaled(scale);

	let glyph_id	= scaled_font.glyph_id('a'); // mono font only
	let column_width = scaled_font.h_advance(glyph_id) as usize;
	let row_height	= (scaled_font.height() + scaled_font.line_gap()) as usize;

	let max_column	= image_out.texture_descriptor.size.width as usize / column_width;
	let max_row		= image_out.texture_descriptor.size.height as usize / row_height;

	from.clamp		(max_column, max_row);
	to.clamp		(max_column, max_row);

	let pixel_size	= image_out.texture_descriptor.format.pixel_size();
	let pixel_color = color.as_linear_rgba_u32().to_le_bytes();

	let table_pos_to_pixel_offset = |column, row| -> usize {
		let x = column * column_width;
		let y = row * row_height;

		(x + y * image_out.texture_descriptor.size.width as usize) * pixel_size
	};

	let pixel_offset_from	= table_pos_to_pixel_offset(from.column, from.row);
	let pixel_offset_to		= table_pos_to_pixel_offset(to.column, to.row) + 4; // + 4 so that last symbol is included

	debug_assert!(pixel_offset_to > pixel_offset_from);
	let range = (pixel_offset_to - pixel_offset_from) / 4;

	debug_assert!((pixel_offset_to - pixel_offset_from) % 4 == 0);
	for pixel_offset in 0 .. range {
		image_out.data[pixel_offset + 0] = pixel_color[2];
		image_out.data[pixel_offset + 1] = pixel_color[1];
		image_out.data[pixel_offset + 2] = pixel_color[0];
		image_out.data[pixel_offset + 3] = pixel_color[3];
	}
}

fn generate_glyph_outline(
	glyph_char	: char,
	font		: &PxScaleFont<&FontVec>,
) -> Option<OutlinedGlyph> {
	let glyph = font.scaled_glyph(glyph_char);
	font.outline_glyph(glyph)
}

pub fn generate_glyph_image(
	glyph_char	: char,
	font		: &Arc<FontVec>,
	scale		: f32,
	color		: Color,
	row			: usize,
	column		: usize,
	alpha_multiplier : f32,
	image_out	: &mut Image,
) {
	// keeping row as it is will render row 0 above visible area because bounds.min.y is negative
	let row = row + 1;

	let scale		= PxScale::from(scale);
    let scaled_font	= font.as_scaled(scale);

	let glyph_id	= scaled_font.glyph_id(glyph_char);

	let column_width = scaled_font.h_advance(glyph_id) as usize;
	let row_height	= (scaled_font.height() + scaled_font.line_gap()) as usize;

	let max_column	= image_out.texture_descriptor.size.width as usize / column_width;
	let max_row		= image_out.texture_descriptor.size.height as usize / row_height;

	if column >= max_column || row >= max_row {
		return;
	}

	let pixel_size	= image_out.texture_descriptor.format.pixel_size();
	let pixel_color = color.as_linear_rgba_u32().to_le_bytes();

	if let Some(glyph_outline) = generate_glyph_outline(glyph_char, &scaled_font) {
		let bounds	= glyph_outline.px_bounds();

		// Draw the glyph into the image per-pixel by using the draw closure
		glyph_outline.draw(|x, y, alpha| {
			let x_woffset = ((x as f32 + bounds.min.x) + (column * column_width) as f32) as usize;
			let y_woffset = ((y as f32 + bounds.min.y) + (row * row_height) as f32) as usize;

			let pixel_offset = (x_woffset + y_woffset * image_out.texture_descriptor.size.width as usize) * pixel_size;

			debug_assert!(image_out.texture_descriptor.format == TextureFormat::Bgra8Unorm);

			let alpha_modified = (alpha * alpha_multiplier).min(1.0);

			image_out.data[pixel_offset + 0] = pixel_color[2];
			image_out.data[pixel_offset + 1] = pixel_color[1];
			image_out.data[pixel_offset + 2] = pixel_color[0];
			image_out.data[pixel_offset + 3] = (alpha_modified * 255.0) as u8;
		});
	}
}

pub fn generate_string_image(
	string		: &String,
	font		: &Arc<FontVec>,
	scale		: f32,
	color		: Color,
	row			: usize,
	column_in	: usize,
	alpha_multiplier : f32,
	image_out	: &mut Image,
) {
	let mut column = column_in;
	for glyph_char in string.chars() {
		debug_assert!(glyph_char != ' ' && glyph_char != '\t');
		generate_glyph_image(glyph_char, font, scale, color, row, column, alpha_multiplier, image_out);
		column += 1;
	}
}

pub struct TablePos {
	pub column	: usize,
	pub row		: usize,
}

impl TablePos {
	pub fn clamp(&mut self, max_column : usize, max_row : usize) {
		self.column = self.column.min(max_column);
		self.row	= self.row.min(max_row);
	}
}
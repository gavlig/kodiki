// default thickness for everything should give us at least ~100 layers to work with
// layers are supposed to be thin enough to visually look like 2d
const THICKNESS					: f32 = 0.001;

mod SURFACE {
	pub const BASE				: f32 = 0.;
	pub const COLORING			: f32 = 1.;
	pub mod HIGHLIGHT {
		pub const GENERIC		: f32 = super::COLORING + 1.;
		pub const DIAGNOSTIC	: f32 = GENERIC + 1.;
		pub const SEARCH		: f32 = DIAGNOSTIC + 1.;
		pub const SELECTION		: f32 = SEARCH + 1.;
	}
	pub const CURSOR			: f32 = HIGHLIGHT::SELECTION + 1.;
	pub const TEXT				: f32 = CURSOR + 1.;
	pub const CHILD_SURFACE		: f32 = TEXT + 2.;
	pub const CENTER_SURFACE	: f32 = CHILD_SURFACE + TEXT + 2.;
	pub const LAST				: f32 = CENTER_SURFACE;
}

mod MINIMAP {
	pub const BASE				: f32 = 0.;
	pub const VIEWPORT			: f32 = BASE + 1.;
	pub const BOOKMARK			: f32 = VIEWPORT + 1.;
	pub const POINTER			: f32 = BOOKMARK + 1.;
	pub const DIFF_GUTTER		: f32 = POINTER + 1.;
	pub mod HIGHLIGHT {
		pub const GENERIC		: f32 = super::POINTER + 1.;
		pub const SELECTION		: f32 = GENERIC + 1.;
		pub const DIAGNOSTIC	: f32 = SELECTION + 1.;
		pub const SEARCH		: f32 = DIAGNOSTIC + 1.;
	}
	pub mod HOVERED_LINE {
		use super::super::SURFACE;

		pub const BASE			: f32 = SURFACE::CENTER_SURFACE + SURFACE::TEXT + 1.0;
		pub const TEXT			: f32 = 1.;
	}
	pub mod BOOKMARK_HINT {
		use super::super::SURFACE;

		pub const BASE			: f32 = SURFACE::TEXT + 1.0;
		pub const TEXT			: f32 = 1.;
	}
}

const RESIZER					: f32 = 0.; // same as main surface
const CONTEXT_SWITCHER			: f32 = SURFACE::LAST + 1.;

pub fn thickness() -> f32 {
	THICKNESS
}

pub fn half_thickness() -> f32 {
	THICKNESS / 2.0
}

fn offset() -> f32 {
	thickness()
}

// Surface

pub mod surface {
	use super :: { offset, SURFACE };

	pub fn base() -> f32 {
		offset() * SURFACE::BASE
	}

	pub fn coloring() -> f32 {
		offset() * SURFACE::COLORING
	}

	pub fn text() -> f32 {
		offset() * SURFACE::TEXT
	}

	pub fn cursor() -> f32 {
		offset() * SURFACE::CURSOR
	}

	pub fn highlight() -> f32 {
		offset() * SURFACE::HIGHLIGHT::GENERIC
	}

	pub fn highlight_selection() -> f32 {
		offset() * SURFACE::HIGHLIGHT::SELECTION
	}

	pub fn highlight_diagnostic() -> f32 {
		offset() * SURFACE::HIGHLIGHT::DIAGNOSTIC
	}

	pub fn highlight_search() -> f32 {
		offset() * SURFACE::HIGHLIGHT::SEARCH
	}

	pub fn child_surface() -> f32 {
		offset() * SURFACE::CHILD_SURFACE
	}

	pub fn center_surface() -> f32 {
		offset() * SURFACE::CENTER_SURFACE
	}

	pub fn last() -> f32 {
		offset() * SURFACE::LAST
	}
}

// Minimap

pub mod minimap {
	use super :: { offset, MINIMAP };

	pub fn viewport() -> f32 {
		offset() * MINIMAP::VIEWPORT
	}

	pub fn pointer() -> f32 {
		offset() * MINIMAP::POINTER
	}

	pub fn bookmark() -> f32 {
		offset() * MINIMAP::BOOKMARK
	}

	pub fn diff_gutter() -> f32 {
		offset() * MINIMAP::DIFF_GUTTER
	}

	pub fn highlight() -> f32 {
		offset() * MINIMAP::HIGHLIGHT::GENERIC
	}

	pub fn highlight_diagnostic() -> f32 {
		offset() * MINIMAP::HIGHLIGHT::DIAGNOSTIC
	}

	pub fn highlight_selection() -> f32 {
		offset() * MINIMAP::HIGHLIGHT::SELECTION
	}

	pub fn highlight_search() -> f32 {
		offset() * MINIMAP::HIGHLIGHT::SEARCH
	}

	pub fn hovered_line() -> f32 {
		offset() * MINIMAP::HOVERED_LINE::BASE
	}

	pub fn hovered_line_text() -> f32 {
		offset() * MINIMAP::HOVERED_LINE::TEXT
	}

	pub fn bookmark_hint() -> f32 {
		offset() * MINIMAP::BOOKMARK_HINT::BASE
	}

	pub fn bookmark_hint_text() -> f32 {
		offset() * MINIMAP::BOOKMARK_HINT::TEXT
	}
}

// Resizer

pub fn resizer() -> f32 {
	offset() * RESIZER
}

// Context Switcher

pub fn context_switcher() -> f32 {
	offset() * CONTEXT_SWITCHER
}
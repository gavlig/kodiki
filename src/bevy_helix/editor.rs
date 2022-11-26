use helix_term::compositor::{Component, Context, Event, EventResult, SurfacesMap, surface_by_id_mut};
use helix_term::keymap::Keymaps;
use helix_term::ui::EditorView;

use helix_core::{
	syntax::{self, HighlightEvent},
	Position,
};

use helix_view::{
	editor::CompleteAction,
	graphics::{CursorKind, Rect},
	input::KeyEvent,
	keyboard::{KeyCode, KeyModifiers},
	Document, Editor, View
};

use helix_tui::buffer::Buffer as Surface;

use helix_term::ui::statusline;

pub struct EditorViewBevy {
	pub editor_view_helix : EditorView,
}

#[derive(Debug, Clone)]
pub enum InsertEvent {
	Key(KeyEvent),
	CompletionApply(CompleteAction),
	TriggerCompletion,
}

impl Default for EditorViewBevy {
	fn default() -> Self {
		Self::new(Keymaps::default())
	}
}

impl EditorViewBevy {
	pub const ID: &'static str = "editor-bevy-component";
	pub const ID_STATUSLINE: &'static str = "statusline-component";
	pub const ID_BUFFERLINE: &'static str = "bufferline-component";
	pub const ID_DIAGNOSTICS: &'static str = "diagnostics-component";

	pub fn new(keymaps: Keymaps) -> Self {
		Self {
			editor_view_helix: EditorView::new(keymaps)
		}
	}

	pub fn render_view(
		&self,
		editor: &Editor,
		doc: &Document,
		view: &View,
		viewport: Rect,
		surfaces: &mut SurfacesMap,
		is_focused: bool,
	) {
		let editor_component_name = String::from(self.id().unwrap());
		let surface_editor = surface_by_id_mut(&editor_component_name, viewport, surfaces);

		// clear with background color
		surface_editor.set_style(viewport, editor.theme.get("ui.background"));

		let inner = view.inner_area();
		let area = view.area;
		let theme = &editor.theme;

		// DAP: Highlight current stack frame position
		let stack_frame = editor.debugger.as_ref().and_then(|debugger| {
			if let (Some(frame), Some(thread_id)) = (debugger.active_frame, debugger.thread_id) {
				debugger
					.stack_frames
					.get(&thread_id)
					.and_then(|bt| bt.get(frame))
			} else {
				None
			}
		});
		if let Some(frame) = stack_frame {
			if doc.path().is_some()
				&& frame
					.source
					.as_ref()
					.and_then(|source| source.path.as_ref())
					== doc.path()
			{
				let line = frame.line - 1; // convert to 0-indexing
				if line >= view.offset.row && line < view.offset.row + area.height as usize {
					surface_editor.set_style(
						Rect::new(
							area.x,
							area.y + (line - view.offset.row) as u16,
							area.width,
							1,
						),
						theme.get("ui.highlight"),
					);
				}
			}
		}

		if is_focused && editor.config().cursorline {
			EditorView::highlight_cursorline(doc, view, surface_editor, theme);
		}
		
		let highlights = EditorView::doc_syntax_highlights(doc, view.offset, inner.height, theme);
		let highlights = syntax::merge(highlights, EditorView::doc_diagnostics_highlights(doc, theme));
		
		let draw_native_cursor = false;
		let highlights: Box<dyn Iterator<Item = HighlightEvent>> =
		if is_focused && draw_native_cursor {
			Box::new(syntax::merge(
				highlights,
				EditorView::doc_selection_highlights(
					editor.mode(),
					doc,
					view,
					theme,
					&editor.config().cursor_shape,
				),
			))
		} else {
			Box::new(highlights)
		};

		EditorView::render_text_highlights(
			doc,
			view.offset,
			inner,
			surface_editor,
			theme,
			highlights,
			&editor.config(),
		);
		EditorView::render_gutter(editor, doc, view, view.area, surface_editor, theme, is_focused);
		EditorView::render_rulers(editor, doc, view, inner, surface_editor, theme);

		if is_focused {
			EditorView::render_focused_view_elements(view, doc, inner, theme, surface_editor);
		}

		let diagnostics_name = String::from(EditorViewBevy::ID_DIAGNOSTICS);
		// let diagnostics_surface = surface_by_id_mut(&diagnostics_name, area, surfaces);

		// self.editor_view_helix.render_diagnostics(doc, view, area, diagnostics_surface, theme);

		let statusline_area = Rect::new(0, 0, area.width, 1);

		let mut context =
			statusline::RenderContext::new(editor, doc, view, is_focused, &self.editor_view_helix.spinners);

		let statusline_name = String::from(EditorViewBevy::ID_STATUSLINE);
		let statusline_surface = surface_by_id_mut(&statusline_name, statusline_area, surfaces);

		statusline::render(&mut context, statusline_area, statusline_surface);
	}
}

impl Component for EditorViewBevy {
	fn handle_event(
		&mut self,
		event: &Event,
		context: &mut Context,
	) -> EventResult {
		self.editor_view_helix.handle_event(event, context)
	}

	fn render(&mut self, area: Rect, surface: &mut Surface, cx: &mut Context) {
		self.editor_view_helix.render(area, surface, cx);
	}

	fn render_ext(&mut self, area: Rect, surfaces: &mut SurfacesMap, cx: &mut Context) {
		let config = cx.editor.config();

		// check if bufferline should be rendered
		// use helix_view::editor::BufferLine;
		// let use_bufferline = match config.bufferline {
		//     BufferLine::Always => true,
		//     BufferLine::Multiple if cx.editor.documents.len() > 1 => true,
		//     _ => false,
		// };

		// -1 for commandline and -1 for bufferline
		// let mut editor_area = area.clip_bottom(1);
		// if use_bufferline {
		//     editor_area = editor_area.clip_top(1);
		// }

		// if the terminal size suddenly changed, we need to trigger a resize
		cx.editor.resize(area);

		// if use_bufferline {
		//     EditorView::render_bufferline(cx.editor, area.with_height(1), surface);
		// }

		for (view, is_focused) in cx.editor.tree.views() {
			let doc = cx.editor.document(view.doc).unwrap();
			self.render_view(cx.editor, doc, view, area, surfaces, is_focused);
		}

		if config.auto_info {
			if let Some(mut info) = cx.editor.autoinfo.take() {
				let auto_info_component_name = String::from("auto-info-component");
				let auto_info_surface = surface_by_id_mut(&auto_info_component_name, area, surfaces); 
				info.render(area, auto_info_surface, cx);
				cx.editor.autoinfo = Some(info)
			}
		}
	}
	
	fn id(&self) -> Option<&'static str> {
		Some(EditorViewBevy::ID)
	}

	fn cursor(&self, area: Rect, editor: &Editor) -> (Option<Position>, CursorKind) {
		self.editor_view_helix.cursor(area, editor)
	}
}

fn canonicalize_key(key: &mut KeyEvent) {
	if let KeyEvent {
		code: KeyCode::Char(_),
		modifiers: _,
	} = key
	{
		key.modifiers.remove(KeyModifiers::SHIFT)
	}
}

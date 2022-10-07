use bevy_debug_text_overlay::screen_print;

use helix_core :: Position;
use helix_view :: graphics :: { CursorKind, Rect };

use helix_tui :: buffer :: Buffer as Surface;

use helix_term :: compositor :: { Compositor, Component, Context, EventResult, Callback, SurfacesMap };

// Cursive-inspired
use helix_term::job::Jobs;
use helix_view::Editor;

pub use helix_view::input::Event;

use std::any::Any;

pub struct CompositorBevy {
	layers: Vec<Box<dyn Component>>,
	last_picker: Option<Box<dyn Component>>,
}

impl CompositorBevy {
	pub fn new() -> anyhow::Result<Self> {
		Ok(Self {
			layers: Vec::new(),
			last_picker: None,
		})
	}
}

impl Compositor for CompositorBevy {
	fn size(&self) -> Rect {
		// self.terminal.size().expect("couldn't get terminal size")
		Rect::default()
	}

	fn resize(&mut self, width: u16, height: u16) {
		// self.terminal
		//     .resize(Rect::new(0, 0, width, height))
		//     .expect("Unable to resize terminal")
	}

	fn save_cursor(&mut self) {
		// if self.terminal.cursor_kind() == CursorKind::Hidden {
		//     self.terminal
		//         .backend_mut()
		//         .show_cursor(CursorKind::Block)
		//         .ok();
		// }
	}

	fn load_cursor(&mut self) {
		// if self.terminal.cursor_kind() == CursorKind::Hidden {
		//     self.terminal.backend_mut().hide_cursor().ok();
		// }
	}

	fn push(&mut self, mut layer: Box<dyn Component>) {
		let size = self.size();
		// trigger required_size on init
		layer.required_size((size.width, size.height));
		self.layers.push(layer);
	}

	fn pop(&mut self) -> Option<Box<dyn Component>> {
		self.layers.pop()
	}

	fn remove(&mut self, id: &'static str) -> Option<Box<dyn Component>> {
		let idx = self
			.layers
			.iter()
			.position(|layer| layer.id() == Some(id))?;
		Some(self.layers.remove(idx))
	}

	fn handle_event(&mut self, event: &Event, cx: &mut Context) -> bool {
		// If it is a key event and a macro is being recorded, push the key event to the recording.
		if let (Event::Key(key), Some((_, keys))) = (event, &mut cx.editor.macro_recording) {
			keys.push(*key);
		}

		let mut callbacks = Vec::new();
		let mut consumed = false;

		// propagate events through the layers until we either find a layer that consumes it or we
		// run out of layers (event bubbling)
		for layer in self.layers.iter_mut().rev() {
			println!("handle_event layer {}", layer.id().unwrap());
			match layer.handle_event(event, cx) {
				EventResult::Consumed(Some(callback)) => {
					callbacks.push(callback);
					consumed = true;
					println!("event consumed with callback");
					break;
				}
				EventResult::Consumed(None) => {
					println!("event consumed None");
					consumed = true;
					break;
				}
				EventResult::Ignored(Some(callback)) => {
					println!("event IGNORED with callback");
					callbacks.push(callback);
				}
				EventResult::Ignored(None) => {
					println!("event IGNORED");
				}
			};
		}

		for callback in callbacks {
			callback(self, cx)
		}

		consumed
	}

	fn render(&mut self, surface: Option<&mut Surface>, cx: &mut Context) {
		let surface = surface.unwrap();
		let area = *surface.area();

		screen_print!("layers: {}", self.layers.len());
		for layer in &mut self.layers {
		    layer.render(area, surface, cx);
		}
	}

	fn render_ext(&mut self, area: Rect, surfaces: &mut SurfacesMap, cx: &mut Context) {
		screen_print!("layers: {}", self.layers.len());
		for layer in &mut self.layers {
		    layer.render_ext(area, surfaces, cx);
		}
	}

	fn cursor(&self, area: Rect, editor: &Editor) -> (Option<Position>, CursorKind) {
		for layer in self.layers.iter().rev() {
			if let (Some(pos), kind) = layer.cursor(area, editor) {
				return (Some(pos), kind);
			}
		}
		(None, CursorKind::Hidden)
	}

	fn has_component(&self, type_name: &str) -> bool {
		self.layers
			.iter()
			.any(|component| component.type_name() == type_name)
	}

	fn find(&mut self, type_name: &str) -> Option<&mut dyn Any>  {
        self.layers
            .iter_mut()
            .find(|component| component.type_name() == type_name)
            .and_then(|component| Some(component.as_any_mut()))
    }

    fn find_id(&mut self, id: &'static str) -> Option<&mut dyn Any> {
        self.layers
            .iter_mut()
            .find(|component| component.id() == Some(id))
            .and_then(|component| Some(component.as_any_mut()))
    }

	fn set_last_picker(&mut self, last_picker: Option<Box<dyn Component>>) {
        self.last_picker = last_picker;
    }

    fn get_last_picker(&mut self) -> &mut Option<Box<dyn Component>> {
        &mut self.last_picker
    }
}
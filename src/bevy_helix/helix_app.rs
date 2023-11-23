use bevy :: prelude :: *;

use arc_swap :: { access::Map, ArcSwap };
use helix_core :: {
	Selection,
	pos_at_coords,
	path		:: get_relative_path,
	diagnostic	:: { NumberOrString, DiagnosticTag },
	syntax		:: { self, HighlightEvent },
};
use helix_lsp :: {
	LspProgressMap,
	lsp,
	util::lsp_pos_to_pos,
};
use helix_view :: {
	current, theme, Editor,
	graphics::Rect, input::Event, tree::Layout,
	doc, doc_mut, DocumentId, Document,
	view, view_mut, View,
	align_view, Align,

	editor		:: { ConfigEvent, EditorEvent, Action },
	document	:: { Mode, DocumentSavedEventResult },
};
use helix_term :: {
	ui, ui::PromptEvent, config::Config, job::Jobs, args::Args, keymap::Keymaps, compositor::Compositor, compositor::SurfacesMap,
	commands, commands::apply_workspace_edit, 
};
use serde_json :: json;

use std	:: {
	path	:: Path,
	sync	:: Arc,
	pin		:: Pin
};

use tokio :: time :: { sleep, Sleep, Duration };

use anyhow :: { Context, Error };

#[cfg(not(windows))]
use {
	signal_hook :: consts::signal,
	signal_hook_tokio :: Signals,
};
#[cfg(windows)]
type Signals = futures_util::stream::Empty<()>;

pub struct HelixApp {
	pub editor	: Editor,
	compositor	: Compositor,
	editor_area	: Rect,
	screen_area	: Rect,

	config		: Arc<ArcSwap<Config>>,

	theme_loader: Arc<theme::Loader>,
	syn_loader  : Arc<syntax::Loader>,

	signals		: Signals,
	jobs		: Jobs,
	lsp_progress: LspProgressMap,

	tokio_idle_timer : Pin<Box<Sleep>>,

	last_doc_id	: DocumentId,
	pub active_document_changed : bool,

	should_render : bool,

	idle_timeout_triggered : bool,
}

impl HelixApp {
	pub fn new(
		args: Args,
		config: Config,
		syn_loader_conf: syntax::Configuration,
		area: Rect
	) -> Result<Self, Error> {
		let mut theme_parent_dirs = vec![helix_loader::config_dir()];
		theme_parent_dirs.extend(helix_loader::runtime_dirs().iter().cloned());
		let theme_loader = std::sync::Arc::new(theme::Loader::new(&theme_parent_dirs));

		let true_color = true;
		let theme = config
			.theme
			.as_ref()
			.and_then(|theme| {
				theme_loader
					.load(theme)
					.map_err(|e| {
						warn!("failed to load theme `{}` - {}", theme, e);
						e
					})
					.ok()
					.filter(|theme| (true_color || theme.is_16_color()))
			})
			.unwrap_or_else(|| theme_loader.default_theme(true_color));

		let syn_loader = std::sync::Arc::new(syntax::Loader::new(syn_loader_conf));

		let mut compositor = Compositor::new(area);
		let config = Arc::new(ArcSwap::from_pointee(config));

		let mut editor = Editor::new(
			helix_view::graphics::Rect::default(),
			theme_loader.clone(),
			syn_loader.clone(),
			Arc::new(Map::new(Arc::clone(&config), |config: &Config| {
				&config.editor
			})),
		);

		let keys = Box::new(Map::new(Arc::clone(&config), |config: &Config| {
			&config.keys
		}));

		let editor_view = Box::new(ui::EditorView::new(Keymaps::new(keys)));
		compositor.push(editor_view);

		if args.load_tutor {
			let path = helix_loader::runtime_file(Path::new("tutor"));
			editor.open(&path, Action::VerticalSplit)?;
			// Unset path to prevent accidentally saving to the original tutor file.
			helix_view::doc_mut!(editor).set_path(None)?;
		} else if !args.files.is_empty() {
			let first = &args.files[0].0; // we know it's not empty
			if first.is_dir() {
				std::env::set_current_dir(&first).context("set current dir")?;
				editor.new_file(Action::VerticalSplit);
				// let picker = ui::file_picker(".".into(), &config.load().editor);
				// compositor.push(Box::new(overlayed(picker)));
			} else {
				let nr_of_files = args.files.len();
				for (i, (file, pos)) in args.files.into_iter().enumerate() {
					if file.is_dir() {
						return Err(anyhow::anyhow!(
							"expected a path to file, found a directory. (to open a directory pass it as first argument)"
						));
					} else {
						// If the user passes in either `--vsplit` or
						// `--hsplit` as a command line argument, all the given
						// files will be opened according to the selected
						// option. If neither of those two arguments are passed
						// in, just load the files normally.
						let action = match args.split {
							_ if i == 0 => Action::VerticalSplit,
							Some(Layout::Vertical) => Action::VerticalSplit,
							Some(Layout::Horizontal) => Action::HorizontalSplit,
							None => Action::Load,
						};
						let doc_id = editor
							.open(&file, action)
							.context(format!("open '{}'", file.to_string_lossy()))?;
						// with Action::Load all documents have the same view
						// NOTE: this isn't necessarily true anymore. If
						// `--vsplit` or `--hsplit` are used, the file which is
						// opened last is focused on.
						let view_id = editor.tree.focus;
						let doc = editor.document_mut(doc_id).unwrap();
						let pos = Selection::point(pos_at_coords(doc.text().slice(..), pos, true));
						doc.set_selection(view_id, pos);
					}
				}
				editor.set_status(format!(
					"Loaded {} file{}.",
					nr_of_files,
					if nr_of_files == 1 { "" } else { "s" } // avoid "Loaded 1 files." grammo
				));
				// align the view to center after all files are loaded,
				// does not affect views without pos since it is at the top
				let (view, doc) = helix_view::current!(editor);
				align_view(doc, view, Align::Center);
			}
		// TODO
		// } else if stdin().is_tty() || cfg!(feature = "integration") {
		//     editor.new_file(Action::VerticalSplit);
		// } else if cfg!(target_os = "macos") {
			// On Linux and Windows, we allow the output of a command to be piped into the new buffer.
			// This doesn't currently work on macOS because of the following issue:
			//   https://github.com/crossterm-rs/crossterm/issues/500
		//	anyhow::bail!("Piping into helix-term is currently not supported on macOS");
		} else {
			// TODO: support stdin?
			//
			// editor
			// 	.new_file_from_stdin(Action::VerticalSplit)
			// 	.unwrap_or_else(|_| editor.new_file(Action::VerticalSplit));

			editor.new_file(Action::VerticalSplit);
			// editor.open(std::path::Path::new("playground/herringbone_spawn.rs"), Action::Load).unwrap();
		}

		editor.set_theme(theme);

		#[cfg(windows)]
		let signals = futures_util::stream::empty();
		#[cfg(not(windows))]
		let signals =
			Signals::new(&[signal::SIGTSTP, signal::SIGCONT]).context("build signal handler")?;

		let doc_id = doc!(editor).id();

		let app = Self {
			compositor,
			editor,
			editor_area: area,
			screen_area: area,

			config,

			theme_loader,
			syn_loader,

			signals,
			jobs: Jobs::new(),
			lsp_progress: LspProgressMap::new(),
			tokio_idle_timer: Box::pin(sleep(Duration::ZERO)),

			last_doc_id: doc_id,
			active_document_changed: false,

			should_render: false,

			idle_timeout_triggered: false,
		};

		Ok(app)
	}

	pub fn should_render(&self) -> bool {
		self.should_render
	}

	pub fn editor_area(&self) -> Rect {
		self.editor_area
	}

	pub fn screen_area(&self) -> Rect {
		self.screen_area
	}

	pub fn resize_editor(&mut self, editor_area: Rect) {
		if self.editor_area == editor_area {
			return;
		}

		self.editor_area = editor_area;
		self.should_render = true;

	}

	pub fn resize_editor_width(&mut self, new_width: u16) {
		if self.editor_area.width == new_width {
			return;
		}

		self.editor_area.width = new_width;
		self.should_render = true;
	}

	pub fn resize_editor_height(&mut self, new_height: u16) {
		if self.editor_area.height == new_height {
			return;
		}

		self.editor_area.height = new_height;
		self.should_render = true;
	}

	pub fn resize_screen_width(&mut self, new_width: u16) {
		if self.screen_area.width == new_width {
			return;
		}

		self.screen_area.width = new_width;
		self.should_render = true;
	}

	pub fn resize_screen(&mut self, screen_area: Rect) {
		if self.screen_area == screen_area {
			return;
		}

		self.screen_area = screen_area;
		self.should_render = true;
	}

	pub fn render(&mut self, surfaces: &mut SurfacesMap) {
		// if !self.should_render {
		// 	for (_name, surface) in surfaces.iter_mut() {
		// 		surface.mark_frozen();
		// 	}

		// 	return;
		// }

		let editor_area = self.editor_area;
		let screen_area = self.screen_area;

		for (name, surface) in surfaces.iter_mut() {
			if name == ui::EditorView::ID {
				surface.resize(editor_area);
			}
			surface.reset();
			surface.mark_clean_and_unfrozen();
		}

		let compositor = &mut self.compositor;

		let mut ctx = helix_term::compositor::ContextExt {
			 vanilla : helix_term::compositor::Context {
				editor	: &mut self.editor,
				jobs	: &mut self.jobs,
				scroll	: None,
			 },
			 surfaces,
			 editor_area,
			 screen_area
		};

		compositor.resize(editor_area);

		compositor.render_ext(&mut ctx);

		let new_doc_id = doc!(self.editor).id();

		self.active_document_changed = new_doc_id != self.last_doc_id;
		self.last_doc_id = new_doc_id;

		self.should_render = false;
	}

	pub fn cursor(&self) -> Option<(Vec<helix_core::Position>, &str)> {
		self.compositor.cursor_ext(&self.editor)
	}

	pub fn editor_focused(&self) -> bool {
		let view = view!(self.editor);

		self.editor.tree.focus == view.id
	}

	pub fn current_doc_len_lines(&self) -> usize {
		let current_doc = doc!(self.editor);
		current_doc.text().len_lines()
	}

	pub fn scroll(&mut self, scroll_offset_in: i32) {
		let mut cx = helix_term::commands::Context {
			editor: &mut self.editor,
			count: None,
			register: None,
			callback: None,
			on_next_key_callback: None,
			jobs: &mut self.jobs,
		};

		use helix_core::movement::Direction;
		let scroll_dir	= if scroll_offset_in < 0 { Direction::Backward } else { Direction::Forward };
		let offset = scroll_offset_in.abs() as usize;

		commands::scroll_viewport_only(&mut cx, offset, scroll_dir);

		self.should_render = true;
	}

	pub fn set_cursor(&mut self, row: usize, column: usize) {
		let (view, doc) = helix_view::current!(self.editor);
		let pos = doc.text().line_to_char(row) + column;

		doc.set_selection(view.id, Selection::point(pos));
	}

	pub fn set_row_offset_internal(&mut self, row: usize) {
		self.set_row_offset_impl(row, false)
	}

	pub fn set_row_offset_external(&mut self, row: usize) {
		self.set_row_offset_impl(row, true)
	}

	fn set_row_offset_impl(&mut self, row: usize, external: bool) {
		let (view, doc) = helix_view::current!(self.editor);
		let doc_text = doc.text().slice(..);

		// always keep external view in sync with internal apart from anchor to accurately represent camera position
		if external {
			view.offset_external = view.offset;
		}

		let Ok(anchor) = doc_text.try_line_to_char(row) else {
			// this can happen when file changed and camera hasn't updated yet due to latency
			return;
		};

		if external {
			view.offset_external.anchor = anchor
		} else {
			view.offset.anchor = anchor;
		}
	}

	pub fn row_offset_internal(&self) -> usize {
		self.row_offset_impl(false)
	}

	pub fn row_offset_external(&self) -> usize {
		self.row_offset_impl(true)
	}

	fn row_offset_impl(&self, external: bool) -> usize {
		let (view, doc) = self.current_ref();
		let doc_text = doc.text().slice(..);
		let anchor = if external { view.offset_external.anchor } else { view.offset.anchor };
		let row_result = doc_text.try_char_to_line(anchor);

		if let Ok(row) = row_result { row } else { doc_text.char_to_line(view.offset.anchor) }
	}

	pub fn enable_inlay_hints(&mut self) {
		self.editor.display_inlay_hints = true;
		let mut ctx = helix_term::commands::Context {
			editor	: &mut self.editor,
			jobs	: &mut self.jobs,

			callback: None,
			count	: None,
			on_next_key_callback: None,
			register: None,
		};
		commands::compute_inlay_hints_for_all_views_ctx(&mut ctx);

		self.should_render = true;
	}

	pub fn disable_inlay_hints(&mut self) {
		self.editor.display_inlay_hints = false;
		let doc = self.current_document_mut();
		doc.reset_all_inlay_hints();

		self.should_render = true;
	}

	pub fn idle_timeout_triggered(&self) -> bool {
		self.idle_timeout_triggered
	}

	pub fn should_close(&self) -> bool {
		self.editor.should_close()
	}

	pub fn close(&mut self) {
		let mut cx = helix_term::compositor::Context {
			editor: &mut self.editor,
			jobs: &mut self.jobs,
			scroll: None,
		};

		if let Err(e) = helix_term::commands::quit(&mut cx, &[], PromptEvent::Validate) {
			cx.editor.set_error(format!("{}", e));
		}
	}

	pub fn current_document_version(&self) -> usize {
		let doc = doc!(self.editor);
		doc.version()
	}

	pub fn current_document_text(&self) -> String {
		let doc = doc!(self.editor);
		doc.text().to_string()
	}

	pub fn current_document(&self) -> &Document {
		doc!(self.editor)
	}

	pub fn current_document_mut(&mut self) -> &mut Document {
		doc_mut!(self.editor)
	}

	pub fn current_view(&self) -> &View {
		view!(self.editor)
	}

	pub fn current_view_mut(&mut self) -> &mut View {
		view_mut!(self.editor)
	}

	pub fn current_ref(&self) -> (&View, &Document) {
		helix_view::current_ref!(self.editor)
	}

	pub fn current_mut(&mut self) -> (&mut View, &mut Document) {
		helix_view::current!(self.editor)
	}

	pub fn current_document_syntax_highlights(&self) -> Box<dyn Iterator<Item = HighlightEvent> + '_> {
		let doc = doc!(self.editor);

		let offset = 0;
		let height = doc.text().len_lines().saturating_sub(1) as u16;
		let theme = &self.editor.theme;

		ui::EditorView::doc_syntax_highlights(doc, offset, height, theme)
	}

	pub fn active_search_pattern(&self) -> Option<String> {
		let search_prompt = self.compositor.find_id::<ui::Prompt>(ui::Prompt::ID);

		if let Some(p) = search_prompt {
			if p.prompt == "search:" && p.line().len() > 0 {
				Some(p.line().clone())
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn gutter_len(&self) -> usize {
		let (view, current_document) = self.current_ref();

		let mut gutter_len = 0 as usize;
		for gutter_type in view.gutters() {
			gutter_len += gutter_type.width(view, current_document);
		}

		gutter_len
	}

	pub fn mode(&self) -> Mode {
		self.editor.mode()
	}

	pub fn hover(&mut self, row: u16, col: u16) {
		let mut ctx = helix_term::commands::Context {
			editor	: &mut self.editor,
			jobs	: &mut self.jobs,

			callback: None,
			count	: None,
			on_next_key_callback: None,
			register: None,
		};

		commands::hover_ext(&mut ctx, row, col);
	}

	pub fn hover_close(&mut self) {
		self.compositor.remove("hover");
	}

	pub async fn jump_to_path(
		&mut self,
		path: &std::path::Path,
		row: Option<usize>,
		col: Option<usize>,
	) {
		{
		    let (view, doc) = current!(self.editor);
		    helix_term::commands::push_jump(view, doc);
		}

	    match self.editor.open(&path, Action::Replace) {
	        Ok(_) => (),
	        Err(err) => {
	            let err = format!("failed to open path: {:?}: {:?}", path, err);
	            self.editor.set_error(err);
	            return;
	        }
	    }

		let (view, doc) = current!(self.editor);

		if let Some(line_index) = row {
			let doc_slice = doc.text().slice(..);
			let mut cursor_pos = doc_slice.line_to_byte(line_index);
			if let Some(column) = col {
				cursor_pos += column;
			}
		    doc.set_selection(view.id, Selection::point(cursor_pos));
		}

	    align_view(doc, view, Align::Center);
	}

	pub fn dark_theme(&self) -> bool {
		self.editor.dark_theme
	}

	pub fn current_document_path(&self) -> Option<&str> {
		let doc = self.current_document();
		if let Some(path) = doc.path() {
			path.to_str()
		} else {
			None
		}
	}

	pub async fn handle_input_event(&mut self, event : &helix_view::input::Event) {
		let mut cx = helix_term::compositor::Context {
			editor: &mut self.editor,
			jobs: &mut self.jobs,
			scroll: None,
		};

		self.should_render |= self.compositor.handle_event(event, &mut cx);
	}

	pub async fn handle_tokio_events(&mut self) {
		self.idle_timeout_triggered = false;

		use futures_util::StreamExt;

		tokio::select! {
			biased;

			Some(signal) = self.signals.next() => {
				self.handle_signals(signal).await;
			}
			Some(callback) = self.jobs.futures.next() => {
				self.jobs.handle_callback(&mut self.editor, &mut self.compositor, callback);
				self.should_render = true;
			}
			Some(callback) = self.jobs.wait_futures.next() => {
				self.jobs.handle_callback(&mut self.editor, &mut self.compositor, callback);
				self.should_render = true;
			}
			event = self.editor.wait_event() => {
				self.handle_editor_event(event).await;
			}
			_ = &mut self.tokio_idle_timer => {
				// if no events don't wait/block at all
			}
		}
	}

	pub fn handle_config_events(&mut self, config_event: ConfigEvent) {
		match config_event {
			ConfigEvent::Refresh => self.refresh_config(),

			// Since only the Application can make changes to Editor's config,
			// the Editor must send up a new copy of a modified config so that
			// the Application can apply it.
			ConfigEvent::Update(editor_config) => {
				let mut app_config = (*self.config.load().clone()).clone();
				app_config.editor = *editor_config;
				self.config.store(Arc::new(app_config));
			}
		}

		// Update all the relevant members in the editor after updating
		// the configuration.
		self.editor.refresh_config();

		// reset view position in case softwrap was enabled/disabled
		let scrolloff = self.editor.config().scrolloff;
		for (view, _) in self.editor.tree.views_mut() {
			let doc = &self.editor.documents[&view.doc];
			view.ensure_cursor_in_view(doc, scrolloff)
		}
	}

	/// refresh language config after config change
	fn refresh_language_config(&mut self) -> Result<(), Error> {
		let syntax_config = helix_core::config::user_syntax_loader()
			.map_err(|err| anyhow::anyhow!("Failed to load language config: {}", err))?;

		self.syn_loader = std::sync::Arc::new(syntax::Loader::new(syntax_config));
		self.editor.syn_loader = self.syn_loader.clone();
		for document in self.editor.documents.values_mut() {
			document.detect_language(self.syn_loader.clone());
		}

		Ok(())
	}

	/// Refresh theme after config change
	fn refresh_theme(&mut self, config: &Config) -> Result<(), Error> {
		let true_color = true;
		let theme = config
			.theme
			.as_ref()
			.and_then(|theme| {
				self.theme_loader
					.load(theme)
					.map_err(|e| {
						log::warn!("failed to load theme `{}` - {}", theme, e);
						e
					})
					.ok()
			})
			.unwrap_or_else(|| self.theme_loader.default_theme(true_color));

		self.editor.set_theme(theme);
		Ok(())
	}

	fn refresh_config(&mut self) {
		let mut refresh_config = || -> Result<(), Error> {
			let default_config = Config::load_default()
				.map_err(|err| anyhow::anyhow!("Failed to load config: {}", err))?;
			self.refresh_language_config()?;
			self.refresh_theme(&default_config)?;
			// Store new config
			self.config.store(Arc::new(default_config));
			Ok(())
		};

		match refresh_config() {
			Ok(_) => {
				self.editor.set_status("Config refreshed");
			}
			Err(err) => {
				self.editor.set_error(err.to_string());
			}
		}
	}

	#[cfg(unix)]
	pub async fn handle_signals(&mut self, signal: i32) {
		match signal {
			signal::SIGTSTP => {
			}
			signal::SIGCONT => {
			}
			signal::SIGUSR1 => {
			}
			_ => unreachable!(),
		}
	}

	pub async fn handle_idle_timeout(&mut self) {
		let mut cx = helix_term::compositor::Context {
			editor: &mut self.editor,
			jobs: &mut self.jobs,
			scroll: None,
		};

		self.should_render |= self.compositor.handle_event(&Event::IdleTimeout, &mut cx);

		self.idle_timeout_triggered = true;
	}

	pub fn handle_document_write(&mut self, doc_save_event: DocumentSavedEventResult) {
		let doc_save_event = match doc_save_event {
			Ok(event) => event,
			Err(err) => {
				self.editor.set_error(err.to_string());
				return;
			}
		};

		let doc = match self.editor.document_mut(doc_save_event.doc_id) {
			None => {
				warn!(
					"received document saved event for non-existent doc id: {}",
					doc_save_event.doc_id
				);

				return;
			}
			Some(doc) => doc,
		};

		debug!(
			"document {:?} saved with revision {}",
			doc.path(),
			doc_save_event.revision
		);

		doc.set_last_saved_revision(doc_save_event.revision);

		let lines = doc_save_event.text.len_lines();
		let bytes = doc_save_event.text.len_bytes();

		if doc.path() != Some(&doc_save_event.path) {
			if let Err(err) = doc.set_path(Some(&doc_save_event.path)) {
				log::error!(
					"error setting path for doc '{:?}': {}",
					doc.path(),
					err.to_string(),
				);

				self.editor.set_error(err.to_string());
				return;
			}

			let loader = self.editor.syn_loader.clone();

			// borrowing the same doc again to get around the borrow checker
			let doc = doc_mut!(self.editor, &doc_save_event.doc_id);
			let id = doc.id();
			doc.detect_language(loader);
			let _ = self.editor.refresh_language_server(id);
		}

		// TODO: fix being overwritten by lsp
		self.editor.set_status(format!(
			"'{}' written, {}L {}B",
			get_relative_path(&doc_save_event.path).to_string_lossy(),
			lines,
			bytes
		));
	}

	#[inline(always)]
	pub async fn handle_editor_event(&mut self, event: EditorEvent) -> bool {
		log::debug!("received editor event: {:?}", event);

		match event {
			EditorEvent::DocumentSaved(event) => {
				self.handle_document_write(event);
				self.should_render = true;
			}
			EditorEvent::ConfigEvent(event) => {
				self.handle_config_events(event);
				self.should_render = true;
			}
			EditorEvent::LanguageServerMessage((id, call)) => {
				self.handle_language_server_message(call, id).await;
				self.should_render = true;
			}
			EditorEvent::DebuggerEvent(payload) => {
				self.should_render |= self.editor.handle_debugger_message(payload).await;
			}
			EditorEvent::IdleTimer => {
				self.editor.clear_idle_timer();
				self.handle_idle_timeout().await;
			}
			EditorEvent::StatusMsgTimer => {
				self.editor.clear_status_msg_timer();
				self.editor.clear_status();
			}
		}

		false
	}

	pub async fn handle_language_server_message(
		&mut self,
		call: helix_lsp::Call,
		server_id: usize,
	) {
		use helix_lsp::{Call, MethodCall, Notification};

		match call {
			Call::Notification(helix_lsp::jsonrpc::Notification { method, params, .. }) => {
				let notification = match Notification::parse(&method, params) {
					Ok(notification) => notification,
					Err(err) => {
						log::error!(
							"received malformed notification from Language Server: {}",
							err
						);
						return;
					}
				};

				match notification {
					Notification::Initialized => {
						let language_server =
							match self.editor.language_servers.get_by_id(server_id) {
								Some(language_server) => language_server,
								None => {
									warn!("can't find language server with id `{}`", server_id);
									return;
								}
							};

						// Trigger a workspace/didChangeConfiguration notification after initialization.
						// This might not be required by the spec but Neovim does this as well, so it's
						// probably a good idea for compatibility.
						if let Some(config) = language_server.config() {
							tokio::spawn(language_server.did_change_configuration(config.clone()));
						}

						let docs = self.editor.documents().filter(|doc| {
							doc.language_server().map(|server| server.id()) == Some(server_id)
						});

						// trigger textDocument/didOpen for docs that are already open
						for doc in docs {
							let url = match doc.url() {
								Some(url) => url,
								None => continue, // skip documents with no path
							};

							let language_id =
								doc.language_id().map(ToOwned::to_owned).unwrap_or_default();

							tokio::spawn(language_server.text_document_did_open(
								url,
								doc.version(),
								doc.text(),
								language_id,
							));
						}
					}
					Notification::PublishDiagnostics(mut params) => {
						let path = match params.uri.to_file_path() {
							Ok(path) => path,
							Err(_) => {
								log::error!("Unsupported file URI: {}", params.uri);
								return;
							}
						};
						let doc = self.editor.document_by_path_mut(&path).filter(|doc| {
							if let Some(version) = params.version {
								if version != doc.version() as i32 {
									log::info!("Version ({version}) is out of date for {path:?} (expected ({}), dropping PublishDiagnostic notification", doc.version());
									return false;
								}
							}

							true
						});

						if let Some(doc) = doc {
							let lang_conf = doc.language_config();
							let text = doc.text();

							let diagnostics = params
								.diagnostics
								.iter()
								.filter_map(|diagnostic| {
									use helix_core::diagnostic::{Diagnostic, Range, Severity::*};
									use lsp::DiagnosticSeverity;

									let language_server = if let Some(language_server) = doc.language_server() {
										language_server
									} else {
										log::warn!("Discarding diagnostic because language server is not initialized: {:?}", diagnostic);
										return None;
									};

									// TODO: convert inside server
									let start = if let Some(start) = lsp_pos_to_pos(
										text,
										diagnostic.range.start,
										language_server.offset_encoding(),
									) {
										start
									} else {
										log::warn!("lsp position out of bounds - {:?}", diagnostic);
										return None;
									};

									let end = if let Some(end) = lsp_pos_to_pos(
										text,
										diagnostic.range.end,
										language_server.offset_encoding(),
									) {
										end
									} else {
										log::warn!("lsp position out of bounds - {:?}", diagnostic);
										return None;
									};

									let severity =
										diagnostic.severity.map(|severity| match severity {
											DiagnosticSeverity::ERROR => Error,
											DiagnosticSeverity::WARNING => Warning,
											DiagnosticSeverity::INFORMATION => Info,
											DiagnosticSeverity::HINT => Hint,
											severity => unreachable!(
												"unrecognized diagnostic severity: {:?}",
												severity
											),
										});

									if let Some(lang_conf) = lang_conf {
										if let Some(severity) = severity {
											if severity < lang_conf.diagnostic_severity {
												return None;
											}
										}
									};

									let code = match diagnostic.code.clone() {
										Some(x) => match x {
											lsp::NumberOrString::Number(x) => {
												Some(NumberOrString::Number(x))
											}
											lsp::NumberOrString::String(x) => {
												Some(NumberOrString::String(x))
											}
										},
										None => None,
									};

									let tags = if let Some(ref tags) = diagnostic.tags {
										let new_tags = tags.iter().filter_map(|tag| {
											match *tag {
												lsp::DiagnosticTag::DEPRECATED => Some(DiagnosticTag::Deprecated),
												lsp::DiagnosticTag::UNNECESSARY => Some(DiagnosticTag::Unnecessary),
												_ => None
											}
										}).collect();

										new_tags
									} else {
										Vec::new()
									};

									Some(Diagnostic {
										range: Range { start, end },
										line: diagnostic.range.start.line as usize,
										message: diagnostic.message.clone(),
										severity,
										code,
										tags,
										source: diagnostic.source.clone(),
										data: diagnostic.data.clone(),
									})
								})
								.collect();

							doc.set_diagnostics(diagnostics);
						}

						// Sort diagnostics first by severity and then by line numbers.
						// Note: The `lsp::DiagnosticSeverity` enum is already defined in decreasing order
						params
							.diagnostics
							.sort_unstable_by_key(|d| (d.severity, d.range.start));

						// Insert the original lsp::Diagnostics here because we may have no open document
						// for diagnosic message and so we can't calculate the exact position.
						// When using them later in the diagnostics picker, we calculate them on-demand.
						self.editor
							.diagnostics
							.insert(params.uri, params.diagnostics);
					}
					Notification::ShowMessage(params) => {
						log::warn!("unhandled window/showMessage: {:?}", params);
					}
					Notification::LogMessage(params) => {
						log::info!("window/logMessage: {:?}", params);
					}
					Notification::ProgressMessage(params)
						if !self
							.compositor
							.has_component(std::any::type_name::<ui::Prompt>()) =>
					{
						let editor_view = self
							.compositor
							.find::<ui::EditorView>()
							.expect("expected at least one EditorView");
						let lsp::ProgressParams { token, value } = params;

						let lsp::ProgressParamsValue::WorkDone(work) = value;
						let parts = match &work {
							lsp::WorkDoneProgress::Begin(lsp::WorkDoneProgressBegin {
								title,
								message,
								percentage,
								..
							}) => (Some(title), message, percentage),
							lsp::WorkDoneProgress::Report(lsp::WorkDoneProgressReport {
								message,
								percentage,
								..
							}) => (None, message, percentage),
							lsp::WorkDoneProgress::End(lsp::WorkDoneProgressEnd { message }) => {
								if message.is_some() {
									(None, message, &None)
								} else {
									self.lsp_progress.end_progress(server_id, &token);
									if !self.lsp_progress.is_progressing(server_id) {
										editor_view.spinners_mut().get_or_create(server_id).stop();
									}
									self.editor.clear_status();

									// we want to render to clear any leftover spinners or messages
									return;
								}
							}
						};

						let token_d: &dyn std::fmt::Display = match &token {
							lsp::NumberOrString::Number(n) => n,
							lsp::NumberOrString::String(s) => s,
						};

						let status = match parts {
							(Some(title), Some(message), Some(percentage)) => {
								format!("[{}] {}% {} - {}", token_d, percentage, title, message)
							}
							(Some(title), None, Some(percentage)) => {
								format!("[{}] {}% {}", token_d, percentage, title)
							}
							(Some(title), Some(message), None) => {
								format!("[{}] {} - {}", token_d, title, message)
							}
							(None, Some(message), Some(percentage)) => {
								format!("[{}] {}% {}", token_d, percentage, message)
							}
							(Some(title), None, None) => {
								format!("[{}] {}", token_d, title)
							}
							(None, Some(message), None) => {
								format!("[{}] {}", token_d, message)
							}
							(None, None, Some(percentage)) => {
								format!("[{}] {}%", token_d, percentage)
							}
							(None, None, None) => format!("[{}]", token_d),
						};

						if let lsp::WorkDoneProgress::End(_) = work {
							self.lsp_progress.end_progress(server_id, &token);
							if !self.lsp_progress.is_progressing(server_id) {
								editor_view.spinners_mut().get_or_create(server_id).stop();
							}
						} else {
							self.lsp_progress.update(server_id, token, work);
						}

						if self.config.load().editor.lsp.display_messages {
							self.editor.set_status(status);
						}
					}
					Notification::ProgressMessage(_params) => {
						// do nothing
					}
					Notification::Exit => {
						self.editor.set_status("Language server exited");

						// Clear any diagnostics for documents with this server open.
						let urls: Vec<_> = self
							.editor
							.documents_mut()
							.filter_map(|doc| {
								if doc.language_server().map(|server| server.id())
									== Some(server_id)
								{
									doc.set_diagnostics(Vec::new());
									doc.url()
								} else {
									None
								}
							})
							.collect();

						for url in urls {
							self.editor.diagnostics.remove(&url);
						}

						// Remove the language server from the registry.
						self.editor.language_servers.remove_by_id(server_id);
					}
				}
			}
			Call::MethodCall(helix_lsp::jsonrpc::MethodCall {
				method, params, id, ..
			}) => {
				let reply = match MethodCall::parse(&method, params) {
					Err(helix_lsp::Error::Unhandled) => {
						error!(
							"Language Server: Method {} not found in request {}",
							method, id
						);
						Err(helix_lsp::jsonrpc::Error {
							code: helix_lsp::jsonrpc::ErrorCode::MethodNotFound,
							message: format!("Method not found: {}", method),
							data: None,
						})
					}
					Err(err) => {
						log::error!(
							"Language Server: Received malformed method call {} in request {}: {}",
							method,
							id,
							err
						);
						Err(helix_lsp::jsonrpc::Error {
							code: helix_lsp::jsonrpc::ErrorCode::ParseError,
							message: format!("Malformed method call: {}", method),
							data: None,
						})
					}
					Ok(MethodCall::WorkDoneProgressCreate(params)) => {
						self.lsp_progress.create(server_id, params.token);

						let editor_view = self
							.compositor
							.find::<ui::EditorView>()
							.expect("expected at least one EditorView");
						let spinner = editor_view.spinners_mut().get_or_create(server_id);
						if spinner.is_stopped() {
							spinner.start();
						}

						Ok(serde_json::Value::Null)
					}
					Ok(MethodCall::ApplyWorkspaceEdit(params)) => {
						let res = apply_workspace_edit(
							&mut self.editor,
							helix_lsp::OffsetEncoding::Utf8,
							&params.edit,
						);

						Ok(json!(lsp::ApplyWorkspaceEditResponse {
							applied: res.is_ok(),
							failure_reason: res.as_ref().err().map(|err| err.kind.to_string()),
							failed_change: res
								.as_ref()
								.err()
								.map(|err| err.failed_change_idx as u32),
						}))
					}
					Ok(MethodCall::WorkspaceFolders) => {
						let language_server =
							self.editor.language_servers.get_by_id(server_id).unwrap();

						Ok(json!(&*language_server.workspace_folders().await))
					}
					Ok(MethodCall::WorkspaceConfiguration(params)) => {
						let result: Vec<_> = params
							.items
							.iter()
							.map(|item| {
								let mut config = match &item.scope_uri {
									Some(scope) => {
										let path = scope.to_file_path().ok()?;
										let doc = self.editor.document_by_path(path)?;
										doc.language_config()?.config.as_ref()?
									}
									None => self
										.editor
										.language_servers
										.get_by_id(server_id)?
										.config()?,
								};
								if let Some(section) = item.section.as_ref() {
									for part in section.split('.') {
										config = config.get(part)?;
									}
								}
								Some(config)
							})
							.collect();
						Ok(json!(result))
					}
					Ok(MethodCall::RegisterCapability(_params)) => {
						log::warn!("Ignoring a client/registerCapability request because dynamic capability registration is not enabled. Please report this upstream to the language server");
						// Language Servers based on the `vscode-languageserver-node` library often send
						// client/registerCapability even though we do not enable dynamic registration
						// for any capabilities. We should send a MethodNotFound JSONRPC error in this
						// case but that rejects the registration promise in the server which causes an
						// exit. So we work around this by ignoring the request and sending back an OK
						// response.

						Ok(serde_json::Value::Null)
					}
				};

				let language_server = match self.editor.language_servers.get_by_id(server_id) {
					Some(language_server) => language_server,
					None => {
						warn!("can't find language server with id `{}`", server_id);
						return;
					}
				};

				tokio::spawn(language_server.reply(id, reply));
			}
			Call::Invalid { id } => log::error!("LSP invalid method call id={:?}", id),
		}
	}
}

use bevy            :: { prelude :: * };

use bevy_debug_text_overlay::screen_print;

use arc_swap        :: { access::Map, ArcSwap };
use futures_util    :: { Stream };
use helix_core      :: {
	config          :: { default_syntax_loader, user_syntax_loader },
	diagnostic      :: { NumberOrString },
	pos_at_coords, syntax, Selection,
};
use helix_lsp       :: { lsp, util :: lsp_pos_to_pos, LspProgressMap };
use helix_view      :: { align_view, editor :: ConfigEvent, theme, tree :: Layout, Align, Editor, graphics :: Rect };
use helix_term      :: { config::Config, job :: Jobs, args::Args, keymap::Keymaps, compositor::Compositor, compositor::SurfacesMap };
use helix_tui 		:: { buffer :: Buffer as Surface };
use serde_json      :: { json };

use std             :: {
	io              :: { stdin, stdout, Write },
	sync            :: { Arc },
	time            :: { Duration, Instant },
};

use anyhow          :: { Context, Error };

#[cfg(not(windows))]
use {
	signal_hook :: { consts::signal, low_level },
	signal_hook_tokio :: { Signals },
};
#[cfg(windows)]
type Signals = futures_util::stream::Empty<()>;

use super :: compositor :: CompositorBevy;
use super :: editor;

const LSP_DEADLINE: Duration = Duration::from_millis(16);

pub struct Application {
	compositor	: CompositorBevy,
	pub editor	: Editor,
	pub area	: Rect,

	config		: Arc<ArcSwap<Config>>,

	theme_loader: Arc<theme::Loader>,
	syn_loader  : Arc<syntax::Loader>,

	signals		: Signals,
	jobs		: Jobs,
	lsp_progress: LspProgressMap,
	last_render : Instant,
}

#[cfg(feature = "integration")]
fn setup_integration_logging() {
//     let level = std::env::var("HELIX_LOG_LEVEL")
//         .map(|lvl| lvl.parse().unwrap())
//         .unwrap_or(log::LevelFilter::Info);

//     // Separate file config so we can include year, month and day in file logs
//     let _ = fern::Dispatch::new()
//         .format(|out, message, record| {
//             out.finish(format_args!(
//                 "{} {} [{}] {}",
//                 chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
//                 record.target(),
//                 record.level(),
//                 message
//             ))
//         })
//         .level(level)
//         .chain(std::io::stdout())
//         .apply();
}

impl Application {
	pub fn new(args: Args, config: Config, area: Rect) -> Result<Self, Error> {
		// #[cfg(feature = "integration")]
		// setup_integration_logging();

		use helix_view::editor::Action;

		let theme_loader = std::sync::Arc::new(theme::Loader::new(
			&helix_loader::config_dir(),
			&helix_loader::runtime_dir(),
		));

		let true_color = true; // config.editor.true_color; // helix_term::true_color();
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

		let syn_loader_conf = user_syntax_loader().unwrap_or_else(|err| {
			eprintln!("Bad language config: {}", err);
			eprintln!("Press <ENTER> to continue with default language config");
			use std::io::Read;
			// This waits for an enter press.
			let _ = std::io::stdin().read(&mut []);
			default_syntax_loader()
		});
		let syn_loader = std::sync::Arc::new(syntax::Loader::new(syn_loader_conf));

		let mut compositor = CompositorBevy::new().context("build compositor")?;
		let config = Arc::new(ArcSwap::from_pointee(config));

		let mut editor = Editor::new(
			helix_view::graphics::Rect::default(),
			theme_loader.clone(),
			syn_loader.clone(),
			Box::new(Map::new(Arc::clone(&config), |config: &Config| {
				&config.editor
			})),
		);

		let keys = Box::new(Map::new(Arc::clone(&config), |config: &Config| {
			&config.keys
		}));

		let editor_view = Box::new(editor::EditorViewBevy::new(Keymaps::new(keys)));
		compositor.push(editor_view);

		if args.load_tutor {
			let path = helix_loader::runtime_dir().join("tutor.txt");
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
				editor.set_status(format!("Loaded {} files.", nr_of_files));
				// align the view to center after all files are loaded,
				// does not affect views without pos since it is at the top
				let (view, doc) = helix_view::current!(editor);
				align_view(doc, view, Align::Center);
			}
		// TODO
		// } else if stdin().is_tty() || cfg!(feature = "integration") {
		//     editor.new_file(Action::VerticalSplit);
		} else if cfg!(target_os = "macos") {
			// On Linux and Windows, we allow the output of a command to be piped into the new buffer.
			// This doesn't currently work on macOS because of the following issue:
			//   https://github.com/crossterm-rs/crossterm/issues/500
			anyhow::bail!("Piping into helix-term is currently not supported on macOS");
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

		let app = Self {
			compositor,
			editor,
			area,

			config,

			theme_loader,
			syn_loader,

			signals,
			jobs: Jobs::new(),
			lsp_progress: LspProgressMap::new(),
			last_render: Instant::now(),
		};

		Ok(app)
	}

	pub fn render(&mut self, surface: &mut Surface) {
		let compositor = &mut self.compositor;

		let mut cx = helix_term::compositor::Context {
			 editor: &mut self.editor,
			 jobs: &mut self.jobs,
			 scroll: None,
		};

		compositor.render(Some(surface), &mut cx);
	}

	pub fn render_ext(&mut self, area: Rect, surfaces: &mut SurfacesMap) {
		let compositor = &mut self.compositor;

		let mut cx = helix_term::compositor::Context {
			 editor: &mut self.editor,
			 jobs: &mut self.jobs,
			 scroll: None,
		};

		compositor.render_ext(area, surfaces, &mut cx);
	}

	pub fn cursor(&mut self, area : helix_view::graphics::Rect) -> (Option<(u16, u16)>, helix_view::graphics::CursorKind) {
		let (pos, kind) = self.compositor.cursor(area, &self.editor);
		(pos.map(|pos| (pos.col as u16, pos.row as u16)), kind)
	}

	pub fn editor_focused(&self) -> bool {
		// hacky, but works for now
		self.compositor.layers.len() == 1
	}

	pub fn handle_input_event(&mut self, event : &helix_view::input::Event) {
		let mut cx = helix_term::compositor::Context {
			editor: &mut self.editor,
			jobs: &mut self.jobs,
			scroll: None,
		};

		let compositor_bevy = &mut self.compositor;
		let compositor_helix = compositor_bevy as &mut dyn helix_term::compositor::Compositor;
		compositor_helix.handle_event(event, &mut cx);
	}

}

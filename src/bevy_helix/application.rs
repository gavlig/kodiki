use bevy            :: { prelude :: * };

use arc_swap        :: { access::Map, ArcSwap };
use helix_core      :: {
	config          :: { default_syntax_loader, user_syntax_loader },
	diagnostic      :: { NumberOrString, DiagnosticTag },
	pos_at_coords, syntax, Selection,
};
use helix_lsp       :: { lsp, util :: lsp_pos_to_pos, LspProgressMap };
use helix_view      :: { align_view, theme, tree :: Layout, Align, Editor, graphics :: Rect, input :: Event };
use helix_term      :: { config::Config, job :: Jobs, args::Args, keymap::Keymaps, compositor::Compositor, compositor::SurfacesMap, ui };
use helix_tui 		:: { buffer :: Buffer as Surface };
use serde_json      :: { json };
use helix_term		:: { commands :: apply_workspace_edit };

use std             :: {
	sync            :: { Arc },
	pin				:: { Pin }
};


use tokio			:: time :: { Sleep, sleep, Duration };

use anyhow          :: { Context, Error };

#[cfg(not(windows))]
use {
	signal_hook :: { consts::signal },
	signal_hook_tokio :: { Signals },
};
#[cfg(windows)]
type Signals = futures_util::stream::Empty<()>;

pub struct Application {
	compositor	: Compositor,
	pub editor	: Editor,
	pub area	: Rect,

	config		: Arc<ArcSwap<Config>>,

	theme_loader: Arc<theme::Loader>,
	syn_loader  : Arc<syntax::Loader>,

	signals		: Signals,
	jobs		: Jobs,
	lsp_progress: LspProgressMap,

	tokio_idle_timer : Pin<Box<Sleep>>,
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

		let syn_loader_conf = user_syntax_loader().unwrap_or_else(|err| {
			eprintln!("Bad language config: {}", err);
			eprintln!("Press <ENTER> to continue with default language config");
			use std::io::Read;
			// This waits for an enter press.
			let _ = std::io::stdin().read(&mut []);
			default_syntax_loader()
		});
		let syn_loader = std::sync::Arc::new(syntax::Loader::new(syn_loader_conf));

		let mut compositor = Compositor::new(area);
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

		let editor_view = Box::new(ui::EditorView::new(Keymaps::new(keys)));
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
			tokio_idle_timer: Box::pin(sleep(Duration::ZERO)),
		};

		Ok(app)
	}

	pub fn render(&mut self, area: Rect, surface: &mut Surface) {
		let compositor = &mut self.compositor;

		let mut cx = helix_term::compositor::Context {
			 editor: &mut self.editor,
			 jobs: &mut self.jobs,
			 scroll: None,
		};
		
		compositor.resize(area);
		compositor.render(area, surface, &mut cx);
	}

	pub fn render_ext(&mut self, area: Rect, surfaces: &mut SurfacesMap) {
		let compositor = &mut self.compositor;

		let mut cx = helix_term::compositor::Context {
			 editor: &mut self.editor,
			 jobs: &mut self.jobs,
			 scroll: None,
		};

		compositor.resize(area);
		compositor.render_ext(area, surfaces, &mut cx);
	}

	pub fn cursor(&mut self, area : Rect) -> (Option<(u16, u16)>, helix_view::graphics::CursorKind) {
		let (pos, kind) = self.compositor.cursor(area, &self.editor);
		(pos.map(|pos| (pos.col as u16, pos.row as u16)), kind)
	}

	pub fn editor_focused(&self) -> bool {
		// hacky, but works for now
		self.compositor.layers.len() == 1
	}

	pub async fn handle_input_event(&mut self, event : &helix_view::input::Event) {
		let mut cx = helix_term::compositor::Context {
			editor: &mut self.editor,
			jobs: &mut self.jobs,
			scroll: None,
		};

		self.compositor.handle_event(event, &mut cx);
	}

	pub async fn handle_tokio_events(&mut self) {
		use futures_util::StreamExt;

		tokio::select! {
			biased;

			Some(signal) = self.signals.next() => {
				self.handle_signals(signal).await;
			}
			Some((id, call)) = self.editor.language_servers.incoming.next() => {
				self.handle_language_server_message(call, id).await;
			}
			Some(payload) = self.editor.debugger_events.next() => {
				let _needs_render = self.editor.handle_debugger_message(payload).await;
			}
			Some(_config_event) = self.editor.config_events.1.recv() => {
				// self.handle_config_events(config_event);
			}
			Some(callback) = self.jobs.futures.next() => {
				self.jobs.handle_callback(&mut self.editor, &mut self.compositor, callback);
			}
			Some(callback) = self.jobs.wait_futures.next() => {
				self.jobs.handle_callback(&mut self.editor, &mut self.compositor, callback);
			}
			_ = &mut self.tokio_idle_timer => {
				// if no events don't wait/block at all
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
		let _should_render = self.compositor.handle_event(&Event::IdleTimeout, &mut cx);
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
						let path = params.uri.to_file_path().unwrap();
						let doc = self.editor.document_by_path_mut(&path);

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
				let call = match MethodCall::parse(&method, params) {
					Ok(call) => call,
					Err(helix_lsp::Error::Unhandled) => {
						error!("Language Server: Method not found {}", method);
						return;
					}
					Err(err) => {
						log::error!(
							"received malformed method call from Language Server: {}: {}",
							method,
							err
						);
						return;
					}
				};

				let reply = match call {
					MethodCall::WorkDoneProgressCreate(params) => {
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
					MethodCall::ApplyWorkspaceEdit(params) => {
						apply_workspace_edit(
							&mut self.editor,
							helix_lsp::OffsetEncoding::Utf8,
							&params.edit,
						);

						Ok(json!(lsp::ApplyWorkspaceEditResponse {
							applied: true,
							failure_reason: None,
							failed_change: None,
						}))
					}
					MethodCall::WorkspaceFolders => {
						let language_server =
							self.editor.language_servers.get_by_id(server_id).unwrap();

						Ok(json!(language_server.workspace_folders()))
					}
					MethodCall::WorkspaceConfiguration(params) => {
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
										.get_by_id(server_id)
										.unwrap()
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

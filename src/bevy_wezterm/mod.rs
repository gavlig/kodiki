use bevy :: prelude :: { * , KeyCode as KeyCodeBevy };
use bevy :: input :: keyboard::KeyboardInput;
use bevy :: gltf :: Gltf;

use bevy_reader_camera :: TextDescriptor;

use crate :: {
	z_order,
	kodiki :: { AppContext, AppMode },
	bevy_ab_glyph :: { ABGlyphFont, ABGlyphFonts, FontAssetHandles },
	kodiki_ui :: {
		KodikiUISystems,
		text_surface			:: { TextSurface, TextSurfaceAnchor, TextSurfacePlacement },
		text_cursor				:: { TextCursor, CursorVisualAsset },
		text_background_quad	:: TextBackgroundQuad,
		raypick					:: RaypickHover,
		resizer					:: Resizer,
	},
};

use portable_pty :: { CommandBuilder, NativePtySystem, MasterPty, PtySize, PtySystem };
use termwiz :: input :: Modifiers as ModifiersWezTerm;
use crossbeam_channel :: unbounded;

use std :: path :: PathBuf;
use std :: sync :: { Arc, Mutex };
use std :: thread;

mod systems;

mod key_code;

use wezterm_portable :: {
	terminalstate :: { TerminalState as WezTermState, TerminalSize },
	color :: ColorPalette,
};

#[derive(Default, Debug)]
pub struct WezTermLiteConfiguration {
	pub color_palette : ColorPalette,
}

impl wezterm_portable::config::TerminalConfiguration for WezTermLiteConfiguration {
    fn color_palette(&self) -> ColorPalette {
        self.color_palette.clone()
    }
}

#[derive(Component)]
pub struct GotoPathHighlight;

#[derive(Component)]
pub struct BevyWezTerm {
	wez_state			: WezTermState,

	active				: bool,
	state_changed		: bool,
	last_rendered_scroll_offset : usize,

	pty_system			: NativePtySystem,
	pty_master			: Mutex<Box<dyn MasterPty>>,
	actions_receiver	: wezterm_portable::mux::ActionsReceiver,

	cursor_entity		: Entity,
	resizer_entity		: Entity,
}

impl BevyWezTerm {
	pub fn spawn(
		name			: &str,
		cwd				: Option<PathBuf>,
		font			: &ABGlyphFont,
		rows			: usize,
		cols			: usize,
		translation		: Option<Vec3>,
		gltf_assets		: &mut Assets<Gltf>,
		cursor_asset	: &mut CursorVisualAsset,
		mesh_assets		: &mut Assets<Mesh>,
		material_assets	: &mut Assets<StandardMaterial>,
		commands		: &mut Commands
	) -> Entity {
		let camera_space	= false;
		let fill_vertically = true;

		let anchor		= TextSurfaceAnchor::Top;
		let top_anchor	= anchor == TextSurfaceAnchor::Top;
		let side_gap	= 0.05;

		let background_entity = TextBackgroundQuad::spawn(
			camera_space,
			fill_vertically,
			top_anchor,
			side_gap,
			font,
			mesh_assets,
			commands
		);

		let cursor_entity = TextCursor::spawn(
			name,
			z_order::surface::cursor(),
			font,
			gltf_assets,
			material_assets,
			cursor_asset,
			commands
		);

		let resizer_entity = Resizer::spawn(
			name,
			UVec2::new(cols as u32, rows as u32),
			mesh_assets,
			material_assets,
			commands
		);

		let terminal_entity = commands.spawn((
			BevyWezTerm::new(
				rows,
				cols,
				cursor_entity,
				resizer_entity,
				cwd
			),
			TextSurface::new(
				name,
				cols,
				TextSurfaceAnchor::Top,
				TextSurfacePlacement::Center,
				Some(background_entity),
			),
			TransformBundle::from_transform(
				Transform::from_translation(translation.unwrap_or(Vec3::ZERO))
			),
			VisibilityBundle::default(),
			RaypickHover::default()
		)).id();

		commands.entity(terminal_entity).push_children(&[cursor_entity, background_entity, resizer_entity]);

		terminal_entity
	}

	pub fn new(
		rows			: usize,
		cols			: usize,
		cursor_entity	: Entity,
		resizer_entity	: Entity,
		cwd				: Option<PathBuf>,
	) -> Self {
		let pty_system = NativePtySystem::default();
		let pty_pair = pty_system
			.openpty(PtySize {
				rows: rows as u16,
				cols: cols as u16,
				pixel_width: 0,
				pixel_height: 0,
			})
			.unwrap();

		let mut cmd = CommandBuilder::new_default_prog_no_login_shell();

		if let Some(cwd) = cwd {
			cmd.cwd(cwd);
		}

		pty_pair.slave.spawn_command(cmd).unwrap(); // TODO: don't crash here, parse result instead

		let pty_writer = pty_pair.master.take_writer().unwrap();

		let pty_reader = pty_pair.master.try_clone_reader().unwrap();

		let (actions_sender, actions_receiver) = unbounded::<wezterm_portable::mux::ActionsVec>();

		thread::spawn(move || wezterm_portable::mux::read_from_pty(actions_sender, pty_reader));

		let state = WezTermState::new(
			TerminalSize::default(),
			Arc::new(WezTermLiteConfiguration::default()),
			"BevyWezTerm",
			"0.1",
			pty_writer
		);

		Self {
			wez_state	: state,
			active		: true,
			state_changed : false,
			last_rendered_scroll_offset : 0,
			pty_system,
			pty_master	: Mutex::new(pty_pair.master),
			actions_receiver,
			cursor_entity,
			resizer_entity
		}
	}

	pub fn resize(
		&mut self,
		rows			: usize,
		cols			: usize,
	) -> bool {
		let new_size = TerminalSize {
			rows,
			cols,
			..default()
		};

		if new_size == self.wez_state.get_size() {
			return false;
		}

		let pty_size = PtySize {
			rows: new_size.rows as u16,
			cols: new_size.cols as u16,
			..default()
		};

		self.pty_master.lock().unwrap().resize(pty_size).unwrap();

		self.wez_state.resize(new_size);

		return true;
	}

	pub fn key_up_down(
		&mut self,
		keyboard_input: &KeyboardInput,
		input_key: &Input<KeyCodeBevy>,
		is_down: bool
	) -> anyhow::Result<()> {
		if let Some(key_code_bevy) = keyboard_input.key_code {
			// ignore modifier only key codes
			match key_code_bevy {
				KeyCodeBevy::LControl | KeyCodeBevy::RControl |
				KeyCodeBevy::LAlt | KeyCodeBevy::RAlt |
				KeyCodeBevy::LShift | KeyCodeBevy::RShift => return Ok(()),
				_ => ()
			};

			// ignore ctrl+1..0 as those are used for context switching
			if input_key.pressed(KeyCode::LControl) || input_key.pressed(KeyCode::RControl) {
				match key_code_bevy {
					KeyCode::Key1 | KeyCode::Key2 | KeyCode::Key3 | KeyCode::Key4 | KeyCode::Key5 |
					KeyCode::Key6 | KeyCode::Key7 | KeyCode::Key8 | KeyCode::Key9 | KeyCode::Key0 => return Ok(()),
					_ => (),
				}
			}
		}

		let modifiers = Self::key_modifiers_bevy_to_wez(input_key);
		let shift_pressed = modifiers.intersects(ModifiersWezTerm::LEFT_SHIFT | ModifiersWezTerm::RIGHT_SHIFT | ModifiersWezTerm::SHIFT);

		let Some(key_code_wez) = Self::keyboard_input_bevy_to_wez(shift_pressed, keyboard_input) else { anyhow::bail!("keyboard_input_bevy_to_wez failed! keyboard_input: {:?}", keyboard_input); };

		self.wez_state.key_up_down(key_code_wez, modifiers, is_down)
	}

	pub fn perform_actions(&mut self) {
		self.state_changed = self.actions_receiver.len() > 0;

		for actions in self.actions_receiver.try_iter() {
			for action in actions.iter() {
				self.wez_state.perform(action.clone());
			}
		}

		if self.state_changed {
			self.wez_state.flush_print();
			self.wez_state.increment_seqno();
		}
	}

	pub fn active(&self) -> bool {
		self.active
	}

	pub fn state_changed(&self) -> bool {
		self.state_changed
	}

	pub fn window_title(&self) -> String {
		let mut window_title = String::from("[");

		let terminal_title = self.wez_state.get_title();
		window_title.push_str(terminal_title);

		window_title.push_str("] - Terminal - Kodiki");

		window_title
	}
}

pub struct BevyWezTermPlugin;

impl Plugin for BevyWezTermPlugin {
	fn build(&self, app: &mut App) {
		app
			// actions are obtained from pty that gets polled in an independent thread
			// we want to poll the buffer of actions in every mode, not just terminal so that buffer doesnt
			// get overflown and in general to keep terminal state always updated regardless if its focused or not
			.add_system(
				systems::update_actions.in_set(OnUpdate(AppMode::Main))
			)
			// all terminal systems chained into a sequence
			// apply_system_buffers + .before(KodikiCommonSystems) guarantees that we avoid desync
			// between setting state for KodikiUI to render and rendering itself
			// TODO: decouple from app
			.add_systems(
				(
					systems::update_text_surface,
					systems::update_resizer,
					systems::update_background_color,
					systems::update_cursor,
					systems::keyboard,
					systems::mouse,
					systems::mouse_goto_path,

					apply_system_buffers
				)
				.chain()
				.before(KodikiUISystems)
				.in_set(OnUpdate(AppContext::Terminal))
			)

			.add_system(
				systems::on_context_switch_out.in_schedule(OnExit(AppContext::Terminal))
			)

			.add_system(
				systems::on_context_switch_in.in_schedule(OnEnter(AppContext::Terminal))
			)
		;
	}
}

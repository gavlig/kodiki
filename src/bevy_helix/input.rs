use bevy :: {
	prelude :: *,
	input :: {
		*,
		keyboard :: *,
		mouse :: *,
	}
};
use helix_view :: keyboard :: { KeyModifiers, KeyCode as KeyCodeHelix };

use tokio :: runtime :: Runtime as TokioRuntime;

use super :: {
	{ BevyHelixSettings, MouseButtonState },
	helix_app :: HelixApp,
};

pub fn keycode_bevy_to_helix(key_code_bevy: KeyCode, shift : bool) -> Option<KeyCodeHelix> {
	let mut success = true;
	let key_code_helix = match key_code_bevy {
		KeyCode::Back		=> KeyCodeHelix::Backspace,
		KeyCode::Return		=> KeyCodeHelix::Enter,
		KeyCode::Left		=> KeyCodeHelix::Left,
		KeyCode::Right		=> KeyCodeHelix::Right,
		KeyCode::Up			=> KeyCodeHelix::Up,
		KeyCode::Down		=> KeyCodeHelix::Down,
		KeyCode::Home		=> KeyCodeHelix::Home,
		KeyCode::End		=> KeyCodeHelix::End,
		KeyCode::PageUp		=> KeyCodeHelix::PageUp,
		KeyCode::PageDown	=> KeyCodeHelix::PageDown,
		KeyCode::Tab		=> KeyCodeHelix::Tab,
		KeyCode::Delete		=> KeyCodeHelix::Delete,
		KeyCode::Insert		=> KeyCodeHelix::Insert,
		KeyCode::Escape		=> KeyCodeHelix::Esc,

		KeyCode::Space		=> KeyCodeHelix::Char(' '),
		KeyCode::Underline	=> KeyCodeHelix::Char('_'),

		KeyCode::Key1		if !shift	=> KeyCodeHelix::Char('1'),
		KeyCode::Key1		if  shift	=> KeyCodeHelix::Char('!'),
		KeyCode::Key2		if !shift	=> KeyCodeHelix::Char('2'),
		KeyCode::Key2		if  shift	=> KeyCodeHelix::Char('@'),
		KeyCode::Key3		if !shift	=> KeyCodeHelix::Char('3'),
		KeyCode::Key3		if  shift	=> KeyCodeHelix::Char('#'),
		KeyCode::Key4		if !shift	=> KeyCodeHelix::Char('4'),
		KeyCode::Key4		if  shift	=> KeyCodeHelix::Char('$'),
		KeyCode::Key5		if !shift	=> KeyCodeHelix::Char('5'),
		KeyCode::Key5		if  shift	=> KeyCodeHelix::Char('%'),
		KeyCode::Key6		if !shift	=> KeyCodeHelix::Char('6'),
		KeyCode::Key6		if  shift	=> KeyCodeHelix::Char('^'),
		KeyCode::Key7		if !shift	=> KeyCodeHelix::Char('7'),
		KeyCode::Key7		if  shift	=> KeyCodeHelix::Char('&'),
		KeyCode::Key8		if !shift	=> KeyCodeHelix::Char('8'),
		KeyCode::Key8		if  shift	=> KeyCodeHelix::Char('*'),
		KeyCode::Key9		if !shift	=> KeyCodeHelix::Char('9'),
		KeyCode::Key9		if  shift	=> KeyCodeHelix::Char('('),
		KeyCode::Key0		if !shift	=> KeyCodeHelix::Char('0'),
		KeyCode::Key0		if  shift	=> KeyCodeHelix::Char(')'),

		KeyCode::A if !shift	=> KeyCodeHelix::Char('a'),
		KeyCode::A if  shift	=> KeyCodeHelix::Char('A'),

		KeyCode::B if !shift	=> KeyCodeHelix::Char('b'),
		KeyCode::B if  shift	=> KeyCodeHelix::Char('B'),

		KeyCode::C if !shift	=> KeyCodeHelix::Char('c'),
		KeyCode::C if  shift	=> KeyCodeHelix::Char('C'),

		KeyCode::D if !shift	=> KeyCodeHelix::Char('d'),
		KeyCode::D if  shift	=> KeyCodeHelix::Char('D'),

		KeyCode::E if !shift	=> KeyCodeHelix::Char('e'),
		KeyCode::E if  shift	=> KeyCodeHelix::Char('E'),

		KeyCode::F if !shift	=> KeyCodeHelix::Char('f'),
		KeyCode::F if  shift	=> KeyCodeHelix::Char('F'),

		KeyCode::G if !shift	=> KeyCodeHelix::Char('g'),
		KeyCode::G if  shift	=> KeyCodeHelix::Char('G'),

		KeyCode::H if !shift	=> KeyCodeHelix::Char('h'),
		KeyCode::H if  shift	=> KeyCodeHelix::Char('H'),

		KeyCode::I if !shift	=> KeyCodeHelix::Char('i'),
		KeyCode::I if  shift	=> KeyCodeHelix::Char('I'),

		KeyCode::J if !shift	=> KeyCodeHelix::Char('j'),
		KeyCode::J if  shift	=> KeyCodeHelix::Char('J'),

		KeyCode::K if !shift	=> KeyCodeHelix::Char('k'),
		KeyCode::K if  shift	=> KeyCodeHelix::Char('K'),

		KeyCode::L if !shift	=> KeyCodeHelix::Char('l'),
		KeyCode::L if  shift	=> KeyCodeHelix::Char('L'),

		KeyCode::M if !shift	=> KeyCodeHelix::Char('m'),
		KeyCode::M if  shift	=> KeyCodeHelix::Char('M'),

		KeyCode::N if !shift	=> KeyCodeHelix::Char('n'),
		KeyCode::N if  shift	=> KeyCodeHelix::Char('N'),

		KeyCode::O if !shift	=> KeyCodeHelix::Char('o'),
		KeyCode::O if  shift	=> KeyCodeHelix::Char('O'),

		KeyCode::P if !shift	=> KeyCodeHelix::Char('p'),
		KeyCode::P if  shift	=> KeyCodeHelix::Char('P'),

		KeyCode::Q if !shift	=> KeyCodeHelix::Char('q'),
		KeyCode::Q if  shift	=> KeyCodeHelix::Char('Q'),

		KeyCode::R if !shift	=> KeyCodeHelix::Char('r'),
		KeyCode::R if  shift	=> KeyCodeHelix::Char('R'),

		KeyCode::S if !shift	=> KeyCodeHelix::Char('s'),
		KeyCode::S if  shift	=> KeyCodeHelix::Char('S'),

		KeyCode::T if !shift	=> KeyCodeHelix::Char('t'),
		KeyCode::T if  shift	=> KeyCodeHelix::Char('T'),

		KeyCode::U if !shift	=> KeyCodeHelix::Char('u'),
		KeyCode::U if  shift	=> KeyCodeHelix::Char('U'),

		KeyCode::V if !shift	=> KeyCodeHelix::Char('v'),
		KeyCode::V if  shift	=> KeyCodeHelix::Char('V'),

		KeyCode::W if !shift	=> KeyCodeHelix::Char('w'),
		KeyCode::W if  shift	=> KeyCodeHelix::Char('W'),

		KeyCode::X if !shift	=> KeyCodeHelix::Char('x'),
		KeyCode::X if  shift	=> KeyCodeHelix::Char('X'),

		KeyCode::Y if !shift	=> KeyCodeHelix::Char('y'),
		KeyCode::Y if  shift	=> KeyCodeHelix::Char('Y'),

		KeyCode::Z if !shift	=> KeyCodeHelix::Char('z'),
		KeyCode::Z if  shift	=> KeyCodeHelix::Char('Z'),

		KeyCode::LBracket	if !shift	=> KeyCodeHelix::Char('['),
		KeyCode::LBracket	if  shift	=> KeyCodeHelix::Char('{'),
		KeyCode::RBracket	if !shift	=> KeyCodeHelix::Char(']'),
		KeyCode::RBracket	if  shift	=> KeyCodeHelix::Char('}'),
		KeyCode::Backslash	if !shift	=> KeyCodeHelix::Char('\\'),
		KeyCode::Backslash	if  shift	=> KeyCodeHelix::Char('|'),
		KeyCode::Semicolon	if !shift	=> KeyCodeHelix::Char(';'),
		KeyCode::Semicolon	if  shift	=> KeyCodeHelix::Char(':'),
		KeyCode::Colon					=> KeyCodeHelix::Char(':'),
		KeyCode::Apostrophe	if !shift	=> KeyCodeHelix::Char('\''),
		KeyCode::Apostrophe	if  shift	=> KeyCodeHelix::Char('"'),

		KeyCode::Comma		if !shift	=> KeyCodeHelix::Char(','),
		KeyCode::Comma		if  shift	=> KeyCodeHelix::Char('<'),
		// KeyCode::Convert	if !shift	=> KeyCodeHelix::Char('.'),
		// KeyCode::Convert	if  shift	=> KeyCodeHelix::Char('>'),
		KeyCode::Slash		if !shift	=> KeyCodeHelix::Char('/'),
		KeyCode::Slash		if  shift	=> KeyCodeHelix::Char('?'),
		KeyCode::Period		if !shift	=> KeyCodeHelix::Char('.'),
		KeyCode::Period		if  shift	=> KeyCodeHelix::Char('>'),
		KeyCode::At			=> KeyCodeHelix::Char('@'),
		KeyCode::Asterisk	=> KeyCodeHelix::Char('*'),
		KeyCode::Plus		=> KeyCodeHelix::Char('+'),
		KeyCode::Minus		if !shift	=> KeyCodeHelix::Char('-'),
		KeyCode::Minus		if  shift	=> KeyCodeHelix::Char('_'),
		KeyCode::Equals		if !shift	=> KeyCodeHelix::Char('='),
		KeyCode::Equals		if  shift	=> KeyCodeHelix::Char('+'),
		KeyCode::Grave		if !shift	=> KeyCodeHelix::Char('`'),
		KeyCode::Grave		if  shift	=> KeyCodeHelix::Char('~'),

		_ => { success = false; KeyCodeHelix::Char('?') }
	};

	if success { Some(key_code_helix) } else { None }
}

pub fn keyboard_input_to_keycode_helix(
	shift : bool,
	keyboard_input : &KeyboardInput
) -> Option<KeyCodeHelix> {
	if keyboard_input.state != ButtonState::Pressed {
		return None;
	}

	let key_code_helix =
	if let Some(key_code_bevy) = keyboard_input.key_code {

		let key_code_helix = keycode_bevy_to_helix(key_code_bevy, shift);
		if key_code_helix.is_none() {
			return None;
		} else {
			key_code_helix.unwrap()
		}

	} else {

		match keyboard_input.scan_code {
			2	=> KeyCodeHelix::Char('!'),
			4	=> KeyCodeHelix::Char('#'),
			5	=> KeyCodeHelix::Char('$'),
			6	=> KeyCodeHelix::Char('%'),
			7	=> KeyCodeHelix::Char('^'),
			8	=> KeyCodeHelix::Char('&'),
			10	=> KeyCodeHelix::Char('('),
			11	=> KeyCodeHelix::Char(')'),
			12	=> KeyCodeHelix::Char('_'),
			13	=> KeyCodeHelix::Char('+'),
			26	=> KeyCodeHelix::Char('{'),
			27	=> KeyCodeHelix::Char('}'),
			40	=> KeyCodeHelix::Char('"'),
			41	=> KeyCodeHelix::Char('~'),
			43	=> KeyCodeHelix::Char('|'),
			51	=> KeyCodeHelix::Char('<'),
			52	=> KeyCodeHelix::Char('>'),
			53	=> KeyCodeHelix::Char('?'),

			_ => { return None; }
		}

	};

	Some(key_code_helix)
}

pub fn key_code_to_helix_modifiers(key : &Input<KeyCode>) -> KeyModifiers {
	let mut modifiers = helix_view::keyboard::KeyModifiers::NONE;

	if key.pressed(KeyCode::LAlt) || key.pressed(KeyCode::RAlt) {
		modifiers.insert(helix_view::keyboard::KeyModifiers::ALT);
	}

	if key.pressed(KeyCode::LControl) || key.pressed(KeyCode::RControl) {
		modifiers.insert(helix_view::keyboard::KeyModifiers::CONTROL);
	}

	if key.pressed(KeyCode::LShift) || key.pressed(KeyCode::RShift) {
		modifiers.insert(helix_view::keyboard::KeyModifiers::SHIFT);
	}

	modifiers
}

pub fn send_keyboard_event(
	keycode 		: &KeyCodeHelix,
	modifiers		: &KeyModifiers,
	tokio_runtime	: &TokioRuntime,
	app				: &mut NonSendMut<HelixApp>,
) {
	let key_event = helix_view::input::KeyEvent {
		code		: *keycode,
		modifiers	: *modifiers,
	};

	let event = helix_view::input::Event::Key(key_event);
	tokio_runtime.block_on(app.handle_input_event(&event));
}

pub fn handle_mouse_events(
	mouse_button	: &Input<MouseButton>,
	mouse_button_state : &MouseButtonState,
	modifiers		: &KeyModifiers,
	column			: u16,
	row				: u16,
	pos_changed		: bool,
	bevy_helix_settings	: &BevyHelixSettings,
	tokio_runtime	: &TokioRuntime,
	app				: &mut NonSendMut<HelixApp>,
) {
	let mut send_mouse_event = |helix_mouse_event_kind: helix_view::input::MouseEventKind| {
		let mouse_event = helix_view::input::MouseEvent {
			column,
			row,
			kind : helix_mouse_event_kind,
			modifiers : *modifiers
		};

		let event = helix_view::input::Event::Mouse(mouse_event);
		tokio_runtime.block_on(app.handle_input_event(&event));
	};

	let bevy2helix_mouse_button = |mouse_button_in: &MouseButton| -> Option<helix_view::input::MouseButton> {
		match mouse_button_in {
			MouseButton::Left	=> Some(helix_view::input::MouseButton::Left),
			MouseButton::Right	=> Some(helix_view::input::MouseButton::Right),
			MouseButton::Middle => Some(helix_view::input::MouseButton::Middle),
			_ => None
		}
	};

	//

	for just_pressed in mouse_button.get_just_pressed() {
		let double_click = mouse_button_state.is_double_click(just_pressed, bevy_helix_settings.double_click_delay_seconds);
		let helix_button = if let Some(btn) = bevy2helix_mouse_button(just_pressed) { btn } else { continue };

		let helix_mouse_event_kind = if double_click {
			helix_view::input::MouseEventKind::DoubleClick(helix_button)
		} else {
			helix_view::input::MouseEventKind::Down(helix_button)
		};

		send_mouse_event(helix_mouse_event_kind);
	}

	for just_released in mouse_button.get_just_released() {
		let helix_mouse_event_kind = helix_view::input::MouseEventKind::Up(
			if let Some(btn) = bevy2helix_mouse_button(just_released) { btn } else { continue }
		);

		send_mouse_event(helix_mouse_event_kind);
	}

	if pos_changed {
		for pressed in mouse_button.get_pressed() {
			if mouse_button.just_pressed(*pressed) {
				continue;
			}

			let helix_mouse_event_kind = helix_view::input::MouseEventKind::Drag(
				if let Some(btn) = bevy2helix_mouse_button(pressed) { btn } else { continue }
			);

			send_mouse_event(helix_mouse_event_kind);
		}
	}
}

pub fn handle_scroll_events(
	scroll_events	: &mut EventReader<MouseWheel>,
	tokio_runtime	: &TokioRuntime,
	app				: &mut NonSendMut<HelixApp>,
) {
	let mut send_mouse_event = |helix_mouse_event_kind: helix_view::input::MouseEventKind| {
		let mouse_event = helix_view::input::MouseEvent {
			column		: 10,
			row			: 10,
			kind		: helix_mouse_event_kind,
			modifiers	: KeyModifiers::NONE
		};

		let event = helix_view::input::Event::Mouse(mouse_event);
		tokio_runtime.block_on(app.handle_input_event(&event));
	};

	// let pixels_per_line = 53.0;
	for scroll_event in scroll_events.iter() {
		match scroll_event.unit {
			MouseScrollUnit::Line => {
				let helix_mouse_event_kind = if scroll_event.y.is_sign_negative() {
					helix_view::input::MouseEventKind::ScrollDown
				} else {
					helix_view::input::MouseEventKind::ScrollUp
				};

				send_mouse_event(helix_mouse_event_kind);
			}
			MouseScrollUnit::Pixel => {
				println!("Scroll (pixel units): vertical: {}, horizontal: {}", scroll_event.y, scroll_event.x);
			}
		}
	}
}
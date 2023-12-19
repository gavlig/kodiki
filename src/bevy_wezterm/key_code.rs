use bevy :: prelude :: { * , KeyCode as KeyCodeBevy };
use bevy :: input :: keyboard :: KeyboardInput;

use termwiz :: input :: { KeyCode as KeyCodeWezTerm, Modifiers as ModifiersWezTerm };

use super :: BevyWezTerm;

impl BevyWezTerm {
	pub fn key_code_bevy_to_wez(key_code_bevy: KeyCodeBevy, shift : bool) -> Option<KeyCodeWezTerm> {
		let mut success = true;
		let key_code_wez = match key_code_bevy {
			KeyCodeBevy::Back		=> KeyCodeWezTerm::Backspace,
			KeyCodeBevy::Return		=> KeyCodeWezTerm::Enter,
			KeyCodeBevy::Left		=> KeyCodeWezTerm::LeftArrow,
			KeyCodeBevy::Right		=> KeyCodeWezTerm::RightArrow,
			KeyCodeBevy::Up			=> KeyCodeWezTerm::UpArrow,
			KeyCodeBevy::Down		=> KeyCodeWezTerm::DownArrow,
			KeyCodeBevy::Home		=> KeyCodeWezTerm::Home,
			KeyCodeBevy::End		=> KeyCodeWezTerm::End,
			KeyCodeBevy::PageUp		=> KeyCodeWezTerm::PageUp,
			KeyCodeBevy::PageDown	=> KeyCodeWezTerm::PageDown,
			KeyCodeBevy::Tab		=> KeyCodeWezTerm::Tab,
			KeyCodeBevy::Delete		=> KeyCodeWezTerm::Delete,
			KeyCodeBevy::Insert		=> KeyCodeWezTerm::Insert,
			KeyCodeBevy::Escape		=> KeyCodeWezTerm::Escape,

			KeyCodeBevy::Space		=> KeyCodeWezTerm::Char(' '),
			KeyCodeBevy::Underline	=> KeyCodeWezTerm::Char('_'),

			KeyCodeBevy::Key1		if !shift	=> KeyCodeWezTerm::Char('1'),
			KeyCodeBevy::Key1		if  shift	=> KeyCodeWezTerm::Char('!'),
			KeyCodeBevy::Key2		if !shift	=> KeyCodeWezTerm::Char('2'),
			KeyCodeBevy::Key2		if  shift	=> KeyCodeWezTerm::Char('@'),
			KeyCodeBevy::Key3		if !shift	=> KeyCodeWezTerm::Char('3'),
			KeyCodeBevy::Key3		if  shift	=> KeyCodeWezTerm::Char('#'),
			KeyCodeBevy::Key4		if !shift	=> KeyCodeWezTerm::Char('4'),
			KeyCodeBevy::Key4		if  shift	=> KeyCodeWezTerm::Char('$'),
			KeyCodeBevy::Key5		if !shift	=> KeyCodeWezTerm::Char('5'),
			KeyCodeBevy::Key5		if  shift	=> KeyCodeWezTerm::Char('%'),
			KeyCodeBevy::Key6		if !shift	=> KeyCodeWezTerm::Char('6'),
			KeyCodeBevy::Key6		if  shift	=> KeyCodeWezTerm::Char('^'),
			KeyCodeBevy::Key7		if !shift	=> KeyCodeWezTerm::Char('7'),
			KeyCodeBevy::Key7		if  shift	=> KeyCodeWezTerm::Char('&'),
			KeyCodeBevy::Key8		if !shift	=> KeyCodeWezTerm::Char('8'),
			KeyCodeBevy::Key8		if  shift	=> KeyCodeWezTerm::Char('*'),
			KeyCodeBevy::Key9		if !shift	=> KeyCodeWezTerm::Char('9'),
			KeyCodeBevy::Key9		if  shift	=> KeyCodeWezTerm::Char('('),
			KeyCodeBevy::Key0		if !shift	=> KeyCodeWezTerm::Char('0'),
			KeyCodeBevy::Key0		if  shift	=> KeyCodeWezTerm::Char(')'),

			KeyCodeBevy::A if !shift	=> KeyCodeWezTerm::Char('a'),
			KeyCodeBevy::A if  shift	=> KeyCodeWezTerm::Char('A'),

			KeyCodeBevy::B if !shift	=> KeyCodeWezTerm::Char('b'),
			KeyCodeBevy::B if  shift	=> KeyCodeWezTerm::Char('B'),

			KeyCodeBevy::C if !shift	=> KeyCodeWezTerm::Char('c'),
			KeyCodeBevy::C if  shift	=> KeyCodeWezTerm::Char('C'),

			KeyCodeBevy::D if !shift	=> KeyCodeWezTerm::Char('d'),
			KeyCodeBevy::D if  shift	=> KeyCodeWezTerm::Char('D'),

			KeyCodeBevy::E if !shift	=> KeyCodeWezTerm::Char('e'),
			KeyCodeBevy::E if  shift	=> KeyCodeWezTerm::Char('E'),

			KeyCodeBevy::F if !shift	=> KeyCodeWezTerm::Char('f'),
			KeyCodeBevy::F if  shift	=> KeyCodeWezTerm::Char('F'),

			KeyCodeBevy::G if !shift	=> KeyCodeWezTerm::Char('g'),
			KeyCodeBevy::G if  shift	=> KeyCodeWezTerm::Char('G'),

			KeyCodeBevy::H if !shift	=> KeyCodeWezTerm::Char('h'),
			KeyCodeBevy::H if  shift	=> KeyCodeWezTerm::Char('H'),

			KeyCodeBevy::I if !shift	=> KeyCodeWezTerm::Char('i'),
			KeyCodeBevy::I if  shift	=> KeyCodeWezTerm::Char('I'),

			KeyCodeBevy::J if !shift	=> KeyCodeWezTerm::Char('j'),
			KeyCodeBevy::J if  shift	=> KeyCodeWezTerm::Char('J'),

			KeyCodeBevy::K if !shift	=> KeyCodeWezTerm::Char('k'),
			KeyCodeBevy::K if  shift	=> KeyCodeWezTerm::Char('K'),

			KeyCodeBevy::L if !shift	=> KeyCodeWezTerm::Char('l'),
			KeyCodeBevy::L if  shift	=> KeyCodeWezTerm::Char('L'),

			KeyCodeBevy::M if !shift	=> KeyCodeWezTerm::Char('m'),
			KeyCodeBevy::M if  shift	=> KeyCodeWezTerm::Char('M'),

			KeyCodeBevy::N if !shift	=> KeyCodeWezTerm::Char('n'),
			KeyCodeBevy::N if  shift	=> KeyCodeWezTerm::Char('N'),

			KeyCodeBevy::O if !shift	=> KeyCodeWezTerm::Char('o'),
			KeyCodeBevy::O if  shift	=> KeyCodeWezTerm::Char('O'),

			KeyCodeBevy::P if !shift	=> KeyCodeWezTerm::Char('p'),
			KeyCodeBevy::P if  shift	=> KeyCodeWezTerm::Char('P'),

			KeyCodeBevy::Q if !shift	=> KeyCodeWezTerm::Char('q'),
			KeyCodeBevy::Q if  shift	=> KeyCodeWezTerm::Char('Q'),

			KeyCodeBevy::R if !shift	=> KeyCodeWezTerm::Char('r'),
			KeyCodeBevy::R if  shift	=> KeyCodeWezTerm::Char('R'),

			KeyCodeBevy::S if !shift	=> KeyCodeWezTerm::Char('s'),
			KeyCodeBevy::S if  shift	=> KeyCodeWezTerm::Char('S'),

			KeyCodeBevy::T if !shift	=> KeyCodeWezTerm::Char('t'),
			KeyCodeBevy::T if  shift	=> KeyCodeWezTerm::Char('T'),

			KeyCodeBevy::U if !shift	=> KeyCodeWezTerm::Char('u'),
			KeyCodeBevy::U if  shift	=> KeyCodeWezTerm::Char('U'),

			KeyCodeBevy::V if !shift	=> KeyCodeWezTerm::Char('v'),
			KeyCodeBevy::V if  shift	=> KeyCodeWezTerm::Char('V'),

			KeyCodeBevy::W if !shift	=> KeyCodeWezTerm::Char('w'),
			KeyCodeBevy::W if  shift	=> KeyCodeWezTerm::Char('W'),

			KeyCodeBevy::X if !shift	=> KeyCodeWezTerm::Char('x'),
			KeyCodeBevy::X if  shift	=> KeyCodeWezTerm::Char('X'),

			KeyCodeBevy::Y if !shift	=> KeyCodeWezTerm::Char('y'),
			KeyCodeBevy::Y if  shift	=> KeyCodeWezTerm::Char('Y'),

			KeyCodeBevy::Z if !shift	=> KeyCodeWezTerm::Char('z'),
			KeyCodeBevy::Z if  shift	=> KeyCodeWezTerm::Char('Z'),

			KeyCodeBevy::BracketLeft	if !shift	=> KeyCodeWezTerm::Char('['),
			KeyCodeBevy::BracketLeft	if  shift	=> KeyCodeWezTerm::Char('{'),
			KeyCodeBevy::BracketRight	if !shift	=> KeyCodeWezTerm::Char(']'),
			KeyCodeBevy::BracketRight	if  shift	=> KeyCodeWezTerm::Char('}'),
			KeyCodeBevy::Backslash	if !shift	=> KeyCodeWezTerm::Char('\\'),
			KeyCodeBevy::Backslash	if  shift	=> KeyCodeWezTerm::Char('|'),
			KeyCodeBevy::Semicolon	if !shift	=> KeyCodeWezTerm::Char(';'),
			KeyCodeBevy::Semicolon	if  shift	=> KeyCodeWezTerm::Char(':'),
			KeyCodeBevy::Colon					=> KeyCodeWezTerm::Char(':'),
			KeyCodeBevy::Apostrophe	if !shift	=> KeyCodeWezTerm::Char('\''),
			KeyCodeBevy::Apostrophe	if  shift	=> KeyCodeWezTerm::Char('"'),

			KeyCodeBevy::Comma		if !shift	=> KeyCodeWezTerm::Char(','),
			KeyCodeBevy::Comma		if  shift	=> KeyCodeWezTerm::Char('<'),
			// KeyCode::Convert	if !shift	=> KeyCodeHelix::Char('.'),
			// KeyCode::Convert	if  shift	=> KeyCodeHelix::Char('>'),
			KeyCodeBevy::Slash		if !shift	=> KeyCodeWezTerm::Char('/'),
			KeyCodeBevy::Slash		if  shift	=> KeyCodeWezTerm::Char('?'),
			KeyCodeBevy::Period		if !shift	=> KeyCodeWezTerm::Char('.'),
			KeyCodeBevy::Period		if  shift	=> KeyCodeWezTerm::Char('>'),
			KeyCodeBevy::At			=> KeyCodeWezTerm::Char('@'),
			KeyCodeBevy::Asterisk	=> KeyCodeWezTerm::Char('*'),
			KeyCodeBevy::Plus		=> KeyCodeWezTerm::Char('+'),
			KeyCodeBevy::Minus		if !shift	=> KeyCodeWezTerm::Char('-'),
			KeyCodeBevy::Minus		if  shift	=> KeyCodeWezTerm::Char('_'),
			KeyCodeBevy::Equals		if !shift	=> KeyCodeWezTerm::Char('='),
			KeyCodeBevy::Equals		if  shift	=> KeyCodeWezTerm::Char('+'),
			KeyCodeBevy::Grave		if !shift	=> KeyCodeWezTerm::Char('`'),
			KeyCodeBevy::Grave		if  shift	=> KeyCodeWezTerm::Char('~'),

			KeyCodeBevy::F1				=> KeyCodeWezTerm::Function(1),
			KeyCodeBevy::F2				=> KeyCodeWezTerm::Function(2),
			KeyCodeBevy::F3				=> KeyCodeWezTerm::Function(3),
			KeyCodeBevy::F4				=> KeyCodeWezTerm::Function(4),
			KeyCodeBevy::F5				=> KeyCodeWezTerm::Function(5),
			KeyCodeBevy::F6				=> KeyCodeWezTerm::Function(6),
			KeyCodeBevy::F7				=> KeyCodeWezTerm::Function(7),
			KeyCodeBevy::F8				=> KeyCodeWezTerm::Function(8),
			KeyCodeBevy::F9				=> KeyCodeWezTerm::Function(9),
			KeyCodeBevy::F10			=> KeyCodeWezTerm::Function(10),
			KeyCodeBevy::F11			=> KeyCodeWezTerm::Function(11),
			KeyCodeBevy::F12			=> KeyCodeWezTerm::Function(12),
			KeyCodeBevy::F13			=> KeyCodeWezTerm::Function(13),
			KeyCodeBevy::F14			=> KeyCodeWezTerm::Function(14),
			KeyCodeBevy::F15			=> KeyCodeWezTerm::Function(15),
			KeyCodeBevy::F16			=> KeyCodeWezTerm::Function(16),
			KeyCodeBevy::F17			=> KeyCodeWezTerm::Function(17),
			KeyCodeBevy::F18			=> KeyCodeWezTerm::Function(18),
			KeyCodeBevy::F19			=> KeyCodeWezTerm::Function(19),
			KeyCodeBevy::F20			=> KeyCodeWezTerm::Function(20),
			KeyCodeBevy::F21			=> KeyCodeWezTerm::Function(21),
			KeyCodeBevy::F22			=> KeyCodeWezTerm::Function(22),
			KeyCodeBevy::F23			=> KeyCodeWezTerm::Function(23),
			KeyCodeBevy::F24			=> KeyCodeWezTerm::Function(24),

			_ => { success = false; KeyCodeWezTerm::Char('?') }
		};

		if success { Some(key_code_wez) } else { None }
	}

	pub fn key_modifiers_bevy_to_wez(key : &Input<KeyCodeBevy>) -> ModifiersWezTerm {
		let mut modifiers = ModifiersWezTerm::NONE;

		for pressed in key.get_pressed() {
			let modifier = match pressed {
				// making left/right distinction breaks wezterm logic, so using generic ctrl,alt,shift instead
				KeyCodeBevy::AltLeft		=> ModifiersWezTerm::ALT,
				KeyCodeBevy::AltRight		=> ModifiersWezTerm::ALT,
				KeyCodeBevy::ControlLeft	=> ModifiersWezTerm::CTRL,
				KeyCodeBevy::ControlRight	=> ModifiersWezTerm::CTRL,
				KeyCodeBevy::ShiftLeft		=> ModifiersWezTerm::SHIFT,
				KeyCodeBevy::ShiftRight		=> ModifiersWezTerm::SHIFT,
				_                        	=> ModifiersWezTerm::NONE,
			};
			modifiers.insert(modifier);
		}

		modifiers
	}

	pub fn keyboard_input_bevy_to_wez(
		shift : bool,
		keyboard_input : &KeyboardInput
	) -> Option<KeyCodeWezTerm> {
		let key_code_wez =
		if let Some(key_code_bevy) = keyboard_input.key_code {

			let key_code_wez = Self::key_code_bevy_to_wez(key_code_bevy, shift);
			if key_code_wez.is_none() {
				return None;
			} else {
				key_code_wez.unwrap()
			}

		} else {

			match keyboard_input.scan_code {
				2	=> KeyCodeWezTerm::Char('!'),
				4	=> KeyCodeWezTerm::Char('#'),
				5	=> KeyCodeWezTerm::Char('$'),
				6	=> KeyCodeWezTerm::Char('%'),
				7	=> KeyCodeWezTerm::Char('^'),
				8	=> KeyCodeWezTerm::Char('&'),
				10	=> KeyCodeWezTerm::Char('('),
				11	=> KeyCodeWezTerm::Char(')'),
				12	=> KeyCodeWezTerm::Char('_'),
				13	=> KeyCodeWezTerm::Char('+'),
				26	=> KeyCodeWezTerm::Char('{'),
				27	=> KeyCodeWezTerm::Char('}'),
				40	=> KeyCodeWezTerm::Char('"'),
				41	=> KeyCodeWezTerm::Char('~'),
				43	=> KeyCodeWezTerm::Char('|'),
				51	=> KeyCodeWezTerm::Char('<'),
				52	=> KeyCodeWezTerm::Char('>'),
				53	=> KeyCodeWezTerm::Char('?'),

				_ => { return None; }
			}

		};

		Some(key_code_wez)
	}
}

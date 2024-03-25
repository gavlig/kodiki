use bevy :: prelude :: { * , KeyCode as KeyCodeBevy };
use bevy :: input :: keyboard :: { KeyboardInput, Key };

use termwiz :: input :: { KeyCode as KeyCodeWezTerm, Modifiers as ModifiersWezTerm };

use super :: BevyWezTerm;

impl BevyWezTerm {
	pub fn key_modifiers_bevy_to_wez(key : &ButtonInput<KeyCodeBevy>) -> ModifiersWezTerm {
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

	pub fn keycode_wez_from_bevy(
		keyboard_input : &KeyboardInput
	) -> Option<KeyCodeWezTerm> {
		let keycode_wez = match &keyboard_input.logical_key {
			Key::Character(smol_str) => KeyCodeWezTerm::Char(smol_str.chars().nth(0).unwrap()),

			Key::Space		=> KeyCodeWezTerm::Char(' '),
			Key::Backspace	=> KeyCodeWezTerm::Backspace,
			Key::Enter		=> KeyCodeWezTerm::Enter,
			Key::ArrowLeft	=> KeyCodeWezTerm::LeftArrow,
			Key::ArrowRight	=> KeyCodeWezTerm::RightArrow,
			Key::ArrowUp	=> KeyCodeWezTerm::UpArrow,
			Key::ArrowDown	=> KeyCodeWezTerm::DownArrow,
			Key::Home		=> KeyCodeWezTerm::Home,
			Key::End		=> KeyCodeWezTerm::End,
			Key::PageUp		=> KeyCodeWezTerm::PageUp,
			Key::PageDown	=> KeyCodeWezTerm::PageDown,
			Key::Tab		=> KeyCodeWezTerm::Tab,
			Key::Delete		=> KeyCodeWezTerm::Delete,
			Key::Insert		=> KeyCodeWezTerm::Insert,
			Key::Escape		=> KeyCodeWezTerm::Escape,
			Key::F1			=> KeyCodeWezTerm::Function(1),
			Key::F2			=> KeyCodeWezTerm::Function(2),
			Key::F3			=> KeyCodeWezTerm::Function(3),
			Key::F4			=> KeyCodeWezTerm::Function(4),
			Key::F5			=> KeyCodeWezTerm::Function(5),
			Key::F6			=> KeyCodeWezTerm::Function(6),
			Key::F7			=> KeyCodeWezTerm::Function(7),
			Key::F8			=> KeyCodeWezTerm::Function(8),
			Key::F9			=> KeyCodeWezTerm::Function(9),
			Key::F10		=> KeyCodeWezTerm::Function(10),
			Key::F11		=> KeyCodeWezTerm::Function(11),
			Key::F12		=> KeyCodeWezTerm::Function(12),
			Key::F13		=> KeyCodeWezTerm::Function(13),
			Key::F14		=> KeyCodeWezTerm::Function(14),
			Key::F15		=> KeyCodeWezTerm::Function(15),
			Key::F16		=> KeyCodeWezTerm::Function(16),
			Key::F17		=> KeyCodeWezTerm::Function(17),
			Key::F18		=> KeyCodeWezTerm::Function(18),
			Key::F19		=> KeyCodeWezTerm::Function(19),
			Key::F20		=> KeyCodeWezTerm::Function(20),
			Key::F21		=> KeyCodeWezTerm::Function(21),
			Key::F22		=> KeyCodeWezTerm::Function(22),
			Key::F23		=> KeyCodeWezTerm::Function(23),
			Key::F24		=> KeyCodeWezTerm::Function(24),
			_				=> KeyCodeWezTerm::Char('?'),
		};

		Some(keycode_wez)
	}
}

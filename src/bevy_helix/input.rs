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
	BevyHelixSettings, MouseButtonState,
	helix_app :: HelixApp,
};

pub fn keycode_helix_from_bevy(
	keyboard_input : &KeyboardInput
) -> Option<KeyCodeHelix> {
	if keyboard_input.state != ButtonState::Pressed {
		return None;
	}

	let keycode_helix = match &keyboard_input.logical_key {
		Key::Character(smol_str) => KeyCodeHelix::Char(smol_str.chars().nth(0).unwrap()),

		Key::Space		=> KeyCodeHelix::Char(' '),
		Key::Backspace	=> KeyCodeHelix::Backspace,
		Key::Enter		=> KeyCodeHelix::Enter,
		Key::ArrowLeft	=> KeyCodeHelix::Left,
		Key::ArrowRight	=> KeyCodeHelix::Right,
		Key::ArrowUp	=> KeyCodeHelix::Up,
		Key::ArrowDown	=> KeyCodeHelix::Down,
		Key::Home		=> KeyCodeHelix::Home,
		Key::End		=> KeyCodeHelix::End,
		Key::PageUp		=> KeyCodeHelix::PageUp,
		Key::PageDown	=> KeyCodeHelix::PageDown,
		Key::Tab		=> KeyCodeHelix::Tab,
		Key::Delete		=> KeyCodeHelix::Delete,
		Key::Insert		=> KeyCodeHelix::Insert,
		Key::Escape		=> KeyCodeHelix::Esc,
		_				=> KeyCodeHelix::Null,
	};

	Some(keycode_helix)
}

pub fn key_code_to_helix_modifiers(key : &ButtonInput<KeyCode>) -> KeyModifiers {
	let mut modifiers = helix_view::keyboard::KeyModifiers::NONE;

	if key.pressed(KeyCode::AltLeft) || key.pressed(KeyCode::AltRight) {
		modifiers.insert(helix_view::keyboard::KeyModifiers::ALT);
	}

	if key.pressed(KeyCode::ControlLeft) || key.pressed(KeyCode::ControlRight) {
		modifiers.insert(helix_view::keyboard::KeyModifiers::CONTROL);
	}

	if key.pressed(KeyCode::ShiftLeft) || key.pressed(KeyCode::ShiftRight) {
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
	mouse_button	: &ButtonInput<MouseButton>,
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

	for scroll_event in scroll_events.read() {
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
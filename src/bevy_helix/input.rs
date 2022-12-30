use bevy :: prelude :: *;
use bevy :: input :: *;
use bevy :: input :: keyboard :: *;
use bevy :: input :: mouse :: *;
use bevy :: ecs :: query :: QueryEntityError;

use crate :: game :: { FontAssetHandles };
use crate :: bevy_ab_glyph :: { ABGlyphFont };

use super :: application :: Application;
use super :: surface :: *;

use helix_view :: keyboard :: { KeyCode as KeyCodeHelix };
use helix_term :: ui :: { EditorView };

use tokio::runtime::Runtime as TokioRuntime;

pub fn keyboard(
	keyboard_events : &mut EventReader<KeyboardInput>,
	key				: &Input<KeyCode>,
	tokio_runtime	: &TokioRuntime,
	app				: &mut NonSendMut<Application>,
) {
	for e in keyboard_events.iter() {
		if e.state != ButtonState::Pressed {
			continue;
		}
		
		let mut modifiers = helix_view::keyboard::KeyModifiers::NONE;
		let mut shift = false;

		if key.pressed(KeyCode::LAlt) || key.pressed(KeyCode::RAlt) {
			modifiers.insert(helix_view::keyboard::KeyModifiers::ALT);
		}

		if key.pressed(KeyCode::LControl) || key.pressed(KeyCode::RControl) {
			modifiers.insert(helix_view::keyboard::KeyModifiers::CONTROL);
		}

		if key.pressed(KeyCode::LShift) || key.pressed(KeyCode::RShift) {
			modifiers.insert(helix_view::keyboard::KeyModifiers::SHIFT);
			shift = true;
		}


		let helix_keycode =
		if e.key_code.is_some() {
		
		match e.key_code.unwrap() {
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

			KeyCode::Key0		=> KeyCodeHelix::Char('0'),
			KeyCode::Key1		=> KeyCodeHelix::Char('1'),
			KeyCode::Key2		=> KeyCodeHelix::Char('2'),
			KeyCode::Key3		=> KeyCodeHelix::Char('3'),
			KeyCode::Key4		=> KeyCodeHelix::Char('4'),
			KeyCode::Key5		=> KeyCodeHelix::Char('5'),
			KeyCode::Key6		=> KeyCodeHelix::Char('6'),
			KeyCode::Key7		=> KeyCodeHelix::Char('7'),
			KeyCode::Key8		=> KeyCodeHelix::Char('8'),
			KeyCode::Key9		=> KeyCodeHelix::Char('9'),

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

			KeyCode::LBracket	=> KeyCodeHelix::Char('['),
			KeyCode::RBracket	=> KeyCodeHelix::Char(']'),
			KeyCode::Backslash	=> KeyCodeHelix::Char('\\'),
			KeyCode::Semicolon	=> KeyCodeHelix::Char(';'),
			KeyCode::Colon		=> KeyCodeHelix::Char(':'),
			KeyCode::Apostrophe	=> KeyCodeHelix::Char('\''),
			
			KeyCode::Comma		=> KeyCodeHelix::Char(','),
			KeyCode::Convert	=> KeyCodeHelix::Char('.'),
			KeyCode::Slash		=> KeyCodeHelix::Char('/'),
			KeyCode::Period		=> KeyCodeHelix::Char('.'),
			KeyCode::At			=> KeyCodeHelix::Char('@'),
			KeyCode::Asterisk	=> KeyCodeHelix::Char('*'),
			KeyCode::Plus		=> KeyCodeHelix::Char('+'),
			KeyCode::Minus		=> KeyCodeHelix::Char('-'),
			KeyCode::Equals		=> KeyCodeHelix::Char('='),
			KeyCode::Grave		=> KeyCodeHelix::Char('`'),

			_ => { println!("skipping keycode {:?}", e.key_code); continue; }
		}
		
		// !e.key_code.is_some()
		} else {

		match e.scan_code {
			2					=> KeyCodeHelix::Char('!'),
			4					=> KeyCodeHelix::Char('#'),
			5					=> KeyCodeHelix::Char('$'),
			6					=> KeyCodeHelix::Char('%'),
			7					=> KeyCodeHelix::Char('^'),
			8					=> KeyCodeHelix::Char('&'),
			10					=> KeyCodeHelix::Char('('),
			11					=> KeyCodeHelix::Char(')'),
			12					=> KeyCodeHelix::Char('_'),
			13					=> KeyCodeHelix::Char('+'),
			26					=> KeyCodeHelix::Char('{'),
			27					=> KeyCodeHelix::Char('}'),
			40					=> KeyCodeHelix::Char('"'),
			41					=> KeyCodeHelix::Char('~'),
			43					=> KeyCodeHelix::Char('|'),
			51					=> KeyCodeHelix::Char('<'),
			52					=> KeyCodeHelix::Char('>'),
			53					=> KeyCodeHelix::Char('?'),
			
			_ => { println!("skipping scancode {:?}", e.scan_code); continue; }
		}

		};

		let key_event = helix_view::input::KeyEvent {
			code : helix_keycode,
			modifiers : modifiers,
		};

		let event = helix_view::input::Event::Key(key_event);
		tokio_runtime.block_on(app.handle_input_event(&event));
	}
}

pub fn mouse(
	mouse_button	: &Input<MouseButton>,
	key				: &Input<KeyCode>,
	scroll_events	: &mut EventReader<MouseWheel>,
	column			: u16,
	row				: u16,
	tokio_runtime	: &TokioRuntime,
	app				: &mut NonSendMut<Application>,
) {
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
	
	let mut make_mouse_event = |helix_mouse_event_kind: helix_view::input::MouseEventKind| {
		let mouse_event = helix_view::input::MouseEvent {
			column		: column,
			row			: row,
			kind		: helix_mouse_event_kind,
			modifiers	: modifiers
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
	
	for just_pressed in mouse_button.get_just_pressed() {
		let helix_mouse_event_kind = helix_view::input::MouseEventKind::Down(
			if let Some(btn) = bevy2helix_mouse_button(just_pressed) { btn } else { continue }
		);
		
		make_mouse_event(helix_mouse_event_kind);
	}
	
	for just_released in mouse_button.get_just_released() {
		let helix_mouse_event_kind = helix_view::input::MouseEventKind::Up(
			if let Some(btn) = bevy2helix_mouse_button(just_released) { btn } else { continue }
		);
		
		make_mouse_event(helix_mouse_event_kind);
	}
	for pressed in mouse_button.get_pressed() {
		let helix_mouse_event_kind = helix_view::input::MouseEventKind::Drag(
			if let Some(btn) = bevy2helix_mouse_button(pressed) { btn } else { continue }
		);
		
		make_mouse_event(helix_mouse_event_kind);
	}
	
	use bevy::input::mouse::MouseScrollUnit;
	// let pixels_per_line = 53.0;
    for scroll_event in scroll_events.iter() {
        match scroll_event.unit {
            MouseScrollUnit::Line => {
				let helix_mouse_event_kind = if scroll_event.y.is_sign_negative() {
					helix_view::input::MouseEventKind::ScrollDown
				} else {
					helix_view::input::MouseEventKind::ScrollUp
				};
				
				make_mouse_event(helix_mouse_event_kind);
            }
            MouseScrollUnit::Pixel => {
                println!("Scroll (pixel units): vertical: {}, horizontal: {}", scroll_event.y, scroll_event.x);
            }
        }
    }
}
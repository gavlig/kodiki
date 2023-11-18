use bevy :: { prelude :: *, window :: CursorGrabMode };

pub fn set_cursor_visibility(v: bool, window: &mut Window) {
	window.cursor.visible = v;
	window.cursor.grab_mode = if v { CursorGrabMode::None } else { CursorGrabMode::Confined };
}

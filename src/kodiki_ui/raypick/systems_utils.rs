use bevy :: prelude :: *;

// Credit to @doomy on discord.
pub fn ray_from_mouse_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec2, Vec3, Vec3) {
    let mouse_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

    let x = 2.0 * (mouse_position.x / window.width() as f32) - 1.0;
    let y = (2.0 * (mouse_position.y / window.height() as f32) - 1.0) * -1.0; // multiplied by -1 because this is all pre bevy 0.11 logic and Window::cursor_position has started being relative to the top left instead of bottom left
	let mouse_position_norm = Vec2::new(x, y);

    let camera_inverse_matrix = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let near = camera_inverse_matrix * Vec3::new(x, y, -1.0).extend(1.0);
    let far = camera_inverse_matrix * Vec3::new(x, y, 1.0).extend(1.0);

    let near = near.truncate() / near.w;
    let far = far.truncate() / far.w;
    let dir: Vec3 = far - near;
    (mouse_position_norm, near, dir)
}
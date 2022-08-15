use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use iyes_loopless	:: { prelude :: * };

use super           :: { * };

pub fn setup_world_system(
	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
		ass				: Res<AssetServer>,
	mut commands		: Commands,
) {
	spawn::camera		(&mut commands);

	spawn::infinite_grid(&mut commands);

	spawn::world_axis	(&mut meshes, &mut materials, &mut commands);

	spawn::file_text	(&ass, &mut commands);
}

pub fn setup_lighting_system(
	mut commands				: Commands,
) {
	const HALF_SIZE: f32		= 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 8000.0,
			// Configure the projection to better fit the scene
			shadow_projection	: OrthographicProjection {
				left			: -HALF_SIZE,
				right			:  HALF_SIZE,
				bottom			: -HALF_SIZE,
				top				:  HALF_SIZE,
				near			: -10.0 * HALF_SIZE,
				far				: 100.0 * HALF_SIZE,
				..default()
			},
			shadows_enabled		: true,
			..default()
		},
		transform				: Transform {
			translation			: Vec3::new(10.0, 2.0, 10.0),
			rotation			: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..default()
		},
		..default()
	});

	// commands
	//     .spawn_bundle(DirectionalLightBundle {
	//         ..Default::default()
	//     })
	//     .insert(Sun); // Marks the light as Sun

	//
}

pub fn setup_camera_system(
	mut query			: Query<&mut FlyCamera>
) {
}

pub fn setup_cursor_visibility_system(
	mut windows	: ResMut<Windows>,
	mut picking	: ResMut<PickingPluginsState>,
) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode	(true);
	window.set_cursor_visibility(false);

	picking.enable_picking 		= false;
	picking.enable_highlighting = false;
	picking.enable_interacting 	= false;
}

pub fn cursor_visibility_system(
	mut windows		: ResMut<Windows>,
	btn				: Res<Input<MouseButton>>,
	key				: Res<Input<KeyCode>>,
	time			: Res<Time>,
	mut q_camera	: Query<&mut FlyCamera>,
		game_mode	: Res<CurrentState<GameMode>>,
	mut picking		: ResMut<PickingPluginsState>,
	mut	commands	: Commands
) {
	let window 		= windows.get_primary_mut().unwrap();
	let cursor_visible = window.cursor_visible();
	let window_id	= window.id();

	let mut set_cursor_visibility = |v| {
		window.set_cursor_visibility(v);
		window.set_cursor_lock_mode(!v);
	};

	let mut set_visibility = |v| {
		set_cursor_visibility(v);

		picking.enable_picking = v;
		picking.enable_highlighting = v;
		picking.enable_interacting = v;

		commands.insert_resource(NextState(
			if v { GameMode::Editor } else { GameMode::InGame }
		));
	};

	if key.just_pressed(KeyCode::Escape) {
		let toggle 	= !cursor_visible;
		set_visibility(toggle);
	}

	if btn.just_pressed(MouseButton::Left) && game_mode.0 == GameMode::InGame{
		set_cursor_visibility(false);
	}

	// #[cfg(debug_assertions)]
	if time.seconds_since_startup() > 1.0 {
		let is_editor = game_mode.0 == GameMode::Editor;
		set_cursor_visibility(is_editor);

		let mut camera 	= q_camera.single_mut();
		camera.enabled_rotation = !is_editor;
	}
}

pub fn input_misc_system(
		btn			: Res<Input<MouseButton>>,
		key			: Res<Input<KeyCode>>,
		time		: Res<Time>,
	mut exit		: EventWriter<AppExit>,
	mut q_camera	: Query<&mut FlyCamera>,
		q_selection	: Query<&Selection>,
		q_children	: Query<&Children>,
) {
	for mut camera in q_camera.iter_mut() {
		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
			let toggle 	= !camera.enabled_follow;
			camera.enabled_follow = toggle;
		}

		if key.just_pressed(KeyCode::Escape) {
			let toggle 	= !camera.enabled_rotation;
			camera.enabled_rotation = toggle;
		}
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}
}

fn check_selection_recursive(
	children	: &Children,
	q_children	: &Query<&Children>,
	q_selection : &Query<&Selection>,
	depth		: u32,
	max_depth 	: u32
 ) -> bool {
	let mut selection_found = false;
	for child in children.iter() {
		let selection = match q_selection.get(*child) {
			Ok(s) => s,
			Err(_) => continue,
		};

		if selection.selected() {
			selection_found = true;
		} else {
			if depth >= max_depth {
				continue;
			}
			let subchildren = q_children.get(*child);
			if subchildren.is_ok() {
				selection_found = check_selection_recursive(subchildren.unwrap(), q_children, q_selection, depth + 1, max_depth);
			}
		}

		if selection_found {
			break;
		}
	}

	selection_found
}

pub fn despawn_system(mut commands: Commands, time: Res<Time>, mut despawn: ResMut<DespawnResource>) {
	if time.seconds_since_startup() > 0.1 {
		for entity in &despawn.entities {
//			println!("Despawning entity {:?}", entity);
			commands.entity(*entity).despawn_recursive();
		}
		despawn.entities.clear();
	}
}
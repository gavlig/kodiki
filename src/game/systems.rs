use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use iyes_loopless	:: { prelude :: * };

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

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

	commands.insert_resource(NextState(AppMode::Main));
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
		// app_mode	: Res<CurrentState<AppMode>>,
	// mut picking		: ResMut<CurrentState<PickingPluginsState>>,
	mut	commands	: Commands
) {
	// let window 		= windows.get_primary_mut();
	// if window.is_none() {
	// 	return;
	// }
	// let window		= window.unwrap();
	// let cursor_visible = window.cursor_visible();
	// let window_id	= window.id();

	// let mut set_cursor_visibility = |v| {
	// 	window.set_cursor_visibility(v);
	// 	window.set_cursor_lock_mode(!v);
	// };

	// let mut set_visibility = |v| {
	// 	set_cursor_visibility(v);

	// 	picking.enable_picking = v;
	// 	picking.enable_highlighting = v;
	// 	picking.enable_interacting = v;

	// 	commands.insert_resource(NextState(
	// 		if v { AppMode::Editor } else { AppMode::Main }
	// 	));
	// };

	// if key.just_pressed(KeyCode::Escape) {
	// 	let toggle 	= !cursor_visible;
	// 	set_visibility(toggle);
	// }

	// if btn.just_pressed(MouseButton::Left) && app_mode.0 == AppMode::Main{
	// 	set_cursor_visibility(false);
	// }

	// // #[cfg(debug_assertions)]
	// if time.seconds_since_startup() > 1.0 {
	// 	let is_editor = app_mode.0 == AppMode::Editor;
	// 	set_cursor_visibility(is_editor);

	// 	let mut camera 	= q_camera.single_mut();
	// 	camera.enabled_rotation = !is_editor;
	// }
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

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key1) {
		// parse_source_file();
	}
}

extern crate rustc_ast;
extern crate rustc_error_messages;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_parse;
extern crate rustc_lexer;
extern crate rustc_data_structures;

use rustc_session::parse::{ ParseSess} ;
use rustc_span::{ FileName, RealFileName };
use rustc_span::edition::Edition;

use rustc_ast::tokenstream::TokenTree;

use std::io::{ Read };

use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}

fn parse_source_file() {
    let source_file	= Some(PathBuf::from("/home/gavlig/workspace/playground/easy_spawn.rs"));
	let load_name 	= file_path_to_string(&source_file);
	let path 		= Path::new(&load_name);
	let display 	= path.display();

	let mut file = match File::open(&path) {
		Err(why) 	=> { println!("couldn't open {}: {}", display, why); return; },
		Ok(file) 	=> file,
	};

	let mut file_content = String::new();
	match file.read_to_string(&mut file_content) {
		Err(why)	=> { println!("couldn't read {}: {}", display, why); return; },
		Ok(_) 		=> println!("Opened file {} for reading", display.to_string()),
	}

    rustc_span::create_session_if_not_set_then(Edition::Edition2021, |_| {
        println!("inside create_session");

        let parser_session = ParseSess::with_silent_emitter(Some(String::from("FATAL MESSAGE AGGHHH")));
        let file_name = FileName::Real(RealFileName::LocalPath(source_file.unwrap()));

        let mut parser = rustc_parse::new_parser_from_source_str(&parser_session, file_name, file_content);

        let tokens = parser.parse_tokens();
        let mut cursor = tokens.trees();

        loop {
            let token_meta = cursor.next();
            if token_meta.is_none() {
                println!("last token parsed! screw you guys i'm going home");
                break;
            }
            let token_meta = token_meta.unwrap();
            match token_meta {
                TokenTree::Token(token, spacing) => {
                    println!("token: {:?} spacing: {:?}\n", token, spacing);
                },
                TokenTree::Delimited(delim_span, delimiter, token_stream) => {
                    println!("delim_span: {:?} delimiter: {:?}\ntoken_stream: {:?}", delim_span, delimiter, token_stream);
                },
            }
        }
    });
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
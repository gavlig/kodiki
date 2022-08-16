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
	let window 		= windows.get_primary_mut();
	if window.is_none() {
		return;
	}
	let window		= window.unwrap();
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

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key1) {
		run_rust_analyzer();
	}
}

use std::io::{self, Write, Read};
use std::process::{Command, Stdio};

use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

use std::{thread, time};

fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}

fn run_rust_analyzer() {
	let source_file	= Some(PathBuf::from("playground/easy_spawn.rs"));
	// let source_file	= Some(PathBuf::from("playground/test_letter_spacing.rs"));
	let load_name 	= file_path_to_string(&source_file);
	let path 		= Path::new(&load_name);
	let display 	= path.display();

	let mut file = match File::open(&path) {
		Err(why) 	=> { println!("couldn't open {}: {}", display, why); return; },
		Ok(file) 	=> file,
	};

	let mut save_content = String::new();
	match file.read_to_string(&mut save_content) {
		Err(why)	=> { println!("couldn't read {}: {}", display, why); return; },
		Ok(_) 		=> println!("Opened file {} for reading", display.to_string()),
	}

	let mut child = Command::new("assets/lsp/rust-analyzer/rust-analyzer")
	.stdin(Stdio::piped())
	.stdout(Stdio::piped())
	// .stderr(Stdio::piped())
	// .env("RA_LOG", "debug")
	.spawn()
	.expect("Failed to spawn child process");
					
	let mut stdin = child.stdin.take().expect("Failed to open stdin");
	let mut stdout = child.stdout.take().expect("Failed to open stdout");
	// let mut stderr = child.stderr.take().expect("Failed to open stderr");
	std::thread::spawn(move || {
		let mut buf = Vec::<u8>::new();
		buf.resize(1024 * 16, 0);

		let mut buf_log = Vec::<u8>::new();
		buf_log.resize(1024 * 1024, 0);

		//
		//

		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "rootPath": "/home/gavlig/workspace/project_gryazevichki/gryazevichki", "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" }, "rust-analyzer.trace.server": "verbose" }}, "id": 1}"#;
		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "rootPath": "/home/gavlig/workspace/project_gryazevichki/gryazevichki", "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" } }}, "id": 1}"#;

		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "workspaceFolders": [{ uri: "file:///home/gavlig/workspace/project_gryazevichki/gryazevichki"}], "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" } }}, "id": 1}"#;
		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "workspaceFolders": [{ uri: "file:///home/gavlig/workspace/fgl_exercise/bevy_fgl_exercise/"}], "capabilities": { "textDocument": { "dynamicRegistration": "true" }, "synchronization": { "dynamicRegistration": "true" } }}, "id": 1}"#;

		#[derive(Serialize, Deserialize, Debug)]
		struct Uri {
			pub uri: &'static str
		}

		#[derive(Serialize, Deserialize, Debug, Clone, Default)]
		struct Synchronization {
			pub dynamicRegistration: bool,
		}

		#[derive(Serialize, Deserialize, Debug, Clone, Default)]
		struct Capabilities {
			pub synchronization: Synchronization,
		}

		#[derive(Serialize, Deserialize, Debug, Clone)]
        struct Params {
            //workspaceFolders: Vec<Uri>,
			pub rootPath: &'static str,
			pub capabilities: Capabilities,
        }

		#[derive(Serialize, Deserialize, Debug, Clone)]
		pub struct Request {
			pub id: i32,
			pub method: &'static str,
			pub params: Params,
		}

		let req = Request {
			id: 1,
			method: "initialize",
			params: Params{ rootPath: "/home/gavlig/workspace/project_gryazevichki/gryazevichki",
			capabilities: Capabilities { synchronization: Synchronization { dynamicRegistration: true } } } };

		#[derive(Serialize)]
        struct JsonRpc {
            jsonrpc: &'static str,
            #[serde(flatten)]
            msg: Request,
        }
        let json = serde_json::to_string(&JsonRpc { jsonrpc: "2.0", msg: req }).unwrap();
		// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "params": { "rootPath": "/home/gavlig/workspace/project_gryazevichki/gryazevichki", "capabilities": { } }, "id": 1}"#;

		let content_length = json.as_bytes().len();
		let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		stdin.flush();

		// println!("\n\n{}\n\nwaiting for output", request);

		thread::sleep(time::Duration::from_millis(5000));

		println!("KODIKI about to read stdout");
		let read_bytes = stdout.read(&mut buf).unwrap();
		println!("KODIKI read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf.as_slice()));
		buf.clear();

		// println!("\nabout to read stderr");
		// let read_bytes = stderr.read(&mut buf_log).unwrap();
		// println!("read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf_log.as_slice()));
		// buf_log.clear();

		//
		//

		println!("KODIKI sending initialized notification");

		let json = r#"{"jsonrpc": "2.0", "method": "initialized", "params": {}}"#;
		let content_length = json.as_bytes().len();
		let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		stdin.flush();

		// 

		thread::sleep(time::Duration::from_millis(1000));

		// println!("\nabout to read stderr3");
		// let read_bytes = stderr.read(&mut buf_log).unwrap();
		// println!("read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf_log.as_slice()));
		// buf_log.clear();

		//
		//

		// let json = format!(r#"{"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": "src/herringbone/spawn.rs", "languageId": "rust", "version": 0, "text": "{}" } }, "id": 3}"#, save_content);

		// let json = r#"{"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": "file:///home/gavlig/workspace/project_gryazevichki/gryazevichki/src/herringbone/spawn.rs", "languageId": "rust", "version": 0, "#;
		// let json = format!("{{\"text\": \"{}\" }} }} }}", save_content);
		// let content_length = json.as_bytes().len();
		// let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		// println!("KODIKI sending didOpen notification");

		// stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		// stdin.flush();

		println!("KODIKI ALL DONE");

		loop {
			thread::sleep(time::Duration::from_millis(3000));
			println!("loop");
		}

		// thread::sleep(time::Duration::from_millis(1000));

		// println!("\nabout to read stderr");
		// let read_bytes = stderr.read(&mut buf_log).unwrap();
		// println!("read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf_log.as_slice()));
		// buf_log.clear();

		//
		//
		
		// let json = r#"{"jsonrpc": "2.0", "method": "textDocument/semanticTokens/full", "params": { "textDocument": { "uri": "file:///src/herringbone/spawn.rs" } }, "id": 4 }"#;
		// let content_length = json.as_bytes().len();
		// let request = format!("Content-Length: {}\r\n\r\n{}", content_length, json);

		// println!("sending highlight request");

		// stdin.write(request.as_bytes()).expect("Failed to write to stdin");
		// stdin.flush();

		// thread::sleep(ten_millis);

		// println!("\nabout to read stdout");
		// let read_bytes = stdout.read(&mut buf).unwrap();
		// println!("read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf.as_slice()));
		// buf.clear();

		// println!("\nabout to read stderr");
		// let read_bytes = stderr.read(&mut buf_log).unwrap();
		// println!("read {} bytes:\n{}", read_bytes, String::from_utf8_lossy(buf_log.as_slice()));
		// buf_log.clear();
	});

				  
	//let output = child.wait_with_output().expect("Failed to read stdout");
    // println!("answer: {}", String::from_utf8_lossy(&output.stdout));

	

	// let mut stdin = child.stdin.take().expect("Failed to open stdin");
	// std::thread::spawn(move || {
	// 	stdin.write(request.as_bytes()).expect("Failed to write to stdin");
	// 	stdin.flush();
	// });

	//  let output = child.wait_with_output().expect("Failed to read stdout");
    // println!("answer2: {}", String::from_utf8_lossy(&output.stdout));
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
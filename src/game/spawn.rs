use bevy				:: prelude :: { * };
use bevy_fly_camera		:: FlyCamera;
use bevy_text_mesh		:: prelude :: { * };

use bevy::render::mesh::shape as render_shape;

use std :: io		:: { prelude :: * };
use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

use super				:: { * };

pub fn camera(
	commands			: &mut Commands
) {
	let camera = commands.spawn_bundle(Camera3dBundle {
			transform: Transform {
				translation: Vec3::new(5., 7., -2.),
				..default()
			},
			..default()
		})
		.insert			(FlyCamera{ yaw : -225.0, pitch : 45.0, enabled_follow : false, ..default() })
		.id				();

	println!			("camera Entity ID {:?}", camera);
}

pub fn ground(
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let ground_size 	= 2000.1;
	let ground_height 	= 0.1;

	let ground			= commands
		.spawn			()
		.insert_bundle	(PbrBundle {
			mesh		: meshes.add(Mesh::from(render_shape::Box::new(ground_size * 2.0, ground_height * 2.0, ground_size * 2.0))),
			material	: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
			transform	: Transform::from_xyz(0.0, -ground_height, 0.0),
			..Default::default()
		})
		.insert			(Transform::from_xyz(0.0, -ground_height, 0.0))
		.insert			(GlobalTransform::default())
		.id				();
		
	println!			("ground Entity ID {:?}", ground);
}

pub fn world_axis(
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands,
) {
	// X
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(1.0, 0.1, 0.1))),
		material	: materials.add			(Color::rgb(0.8, 0.1, 0.1).into()),
		transform	: Transform::from_xyz	(0.5, 0.0 + 0.05, 0.0),
		..Default::default()
	});
	// Y
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(0.1, 1.0, 0.1))),
		material	: materials.add			(Color::rgb(0.1, 0.8, 0.1).into()),
		transform	: Transform::from_xyz	(0.0, 0.5 + 0.05, 0.0),
		..Default::default()
	});
	// Z
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(0.1, 0.1, 1.0))),
		material	: materials.add			(Color::rgb(0.1, 0.1, 0.8).into()),
		transform	: Transform::from_xyz	(0.0, 0.0 + 0.05, 0.5),
		..Default::default()
	});
}

pub fn fixed_cube(
	pose				: Transform,
	hsize				: Vec3,
	color				: Color,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	commands.spawn_bundle(PbrBundle {
		mesh			: meshes.add	(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0))),
		material		: materials.add	(color.into()),
		..default()
	})
	.insert				(pose)
	.insert				(GlobalTransform::default());
}

pub fn text_mesh(
	text_in				: &String,
	ass					: &Res<AssetServer>,
	commands			: &mut Commands,
) {
    let font: Handle<TextMeshFont> = ass.load("fonts/FiraMono-Medium.ttf");

    commands.spawn_bundle(TextMeshBundle {
        text_mesh: TextMesh {
            text: text_in.clone(),
            style: TextMeshStyle {
                font: font.clone(),
                font_size: SizeUnit::NonStandard(9.),
                color: Color::rgb(0.0, 0.0, 0.0),
                ..Default::default()
            },
            size: TextMeshSize {
                ..Default::default()
            },
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(-1., 1.75, 0.),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}

pub fn file_text(
	ass					: &Res<AssetServer>,
	commands			: &mut Commands
) {
	let source_file	= Some(PathBuf::from("playground/easy_spawn.rs"));
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

	text_mesh		(&save_content, ass, commands);
}
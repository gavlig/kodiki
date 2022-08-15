use bevy				:: prelude :: { * };
use bevy_fly_camera		:: FlyCamera;
use bevy_prototype_lyon::prelude::tess::FillGeometryBuilder;
use bevy_text_mesh		:: prelude :: { * };
use bevy_prototype_lyon	:: prelude :: { * };

use bevy::render::mesh::shape as render_shape;

use std :: io		:: { prelude :: * };
use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

extern crate freetype;
use freetype as ft;
use freetype :: Library;
use freetype :: face :: LoadFlag;

use super				:: { * };

pub fn camera(
	commands			: &mut Commands
) {
	let camera = commands.spawn_bundle(Camera2dBundle::default());

	// let camera = commands.spawn_bundle(Camera3dBundle {
	// 		transform: Transform {
	// 			translation: Vec3::new(5., 7., -2.),
	// 			..default()
	// 		},
	// 		..default()
	// 	})
	// 	.insert			(FlyCamera{ yaw : -225.0, pitch : 45.0, enabled_follow : false, ..default() })
	// 	.id				();

	// println!			("camera Entity ID {:?}", camera);
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
	y					: f32,
	ass					: &Res<AssetServer>,
	commands			: &mut Commands,
) {
    let font: Handle<TextMeshFont> = ass.load("fonts/droidsans-mono.ttf"); //("fonts/FiraMono-Medium.ttf");

    commands.spawn_bundle(TextMeshBundle {
        text_mesh: TextMesh {
            text: text_in.clone(),
            style: TextMeshStyle {
                font: font.clone(),
                font_size: SizeUnit::NonStandard(9.),
                color: Color::rgb(0.2, 0.2, 0.2),
                ..Default::default()
            },
            size: TextMeshSize {
                ..Default::default()
            },
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(-1., y, 0.),
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

fn draw_curve(curve: ft::outline::Curve) {
    match curve {
        ft::outline::Curve::Line(pt) =>
            println!("L {} {}", pt.x, -pt.y),
        ft::outline::Curve::Bezier2(pt1, pt2) =>
            println!("Q {} {} {} {}", pt1.x, -pt1.y, pt2.x, -pt2.y),
        ft::outline::Curve::Bezier3(pt1, pt2, pt3) =>
            println!("C {} {} {} {} {} {}", pt1.x, -pt1.y,
                                            pt2.x, -pt2.y,
                                            pt3.x, -pt3.y)
    }
}

pub fn file_text(
	ass					: &Res<AssetServer>,
	commands			: &mut Commands
) {
	// Init the library
    let lib = Library::init().unwrap();
    // Load a font face
    let face = lib.new_face("assets/fonts/droidsans-mono.ttf", 0).unwrap();
    // Set the font size
    face.set_char_size(40 * 8, 0, 50, 0).unwrap();
    // Load a character
    // face.load_char('A' as usize, LoadFlag::RENDER).unwrap();
    // // Get the glyph instance
    // let glyph = face.glyph();
    
    // let metrics = glyph.metrics();
    // let xmin = metrics.horiBearingX - 5;
    // let width = metrics.width + 10;
    // let ymin = -metrics.horiBearingY - 5;
    // let height = metrics.height + 10;
    // let outline = glyph.outline().unwrap();

	// for contour in outline.contours_iter() {
    //     let start = contour.start();
    //     for curve in contour {
    //         draw_curve(curve);
    //     }
    // }

	let source_file	= Some(PathBuf::from("playground/test_simple.rs"));
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

	let mut lines	= save_content.lines();
	let mut y		= 5.0;
	loop {
		let line 	= match lines.next() {
			Some(l)	=> l,
			None	=> break,
		};
		
		let mut x	= 0.0;
		for char in line.chars() {
			println!("char: {}", char);

			face.load_char(char as usize, LoadFlag::NO_SCALE).unwrap();
			// Get the glyph instance
			let glyph = face.glyph();
			let outline = glyph.outline().unwrap();
			
			let metrics = glyph.metrics();

			println!("horiAdvance: {}", metrics.horiAdvance);

			let mut path_builder = PathBuilder::new();
			let coef = 1.0 / 20.0;
			
			for contour in outline.contours_iter() {
			    let start = contour.start();
				path_builder.move_to(Vec2::new(start.x as f32, start.y as f32) * coef);
			    for curve in contour {
			        match curve {
						ft::outline::Curve::Line(pt) => {
							// println!("L {} {}", pt.x, pt.y);
							let to = Vec2::new(pt.x as f32, pt.y as f32) * coef;
							path_builder.line_to(to);
						},
						ft::outline::Curve::Bezier2(pt1, pt2) => {
							// println!("Q {} {} {} {}", pt1.x, pt1.y, pt2.x, -pt2.y);
							let ctrl = Vec2::new(pt1.x as f32, pt1.y as f32) * coef;
							let to = Vec2::new(pt2.x as f32, pt2.y as f32) * coef;
							path_builder.quadratic_bezier_to(ctrl, to);
						},
						ft::outline::Curve::Bezier3(pt1, pt2, pt3) => {
							// println!("C {} {} {} {} {} {}", pt1.x, pt1.y,
							// 								pt2.x, pt2.y,
							// 								pt3.x, pt3.y);
							let ctrl1 = Vec2::new(pt1.x as f32, pt1.y as f32) * coef;
							let ctrl2 = Vec2::new(pt2.x as f32, pt2.y as f32) * coef;
							let to = Vec2::new(pt3.x as f32, pt3.y as f32) * coef;
							path_builder.cubic_bezier_to(ctrl1, ctrl2, to);
						},
					}
			    }
			}

			let line = path_builder.build();

			commands.spawn_bundle(GeometryBuilder::build_as(
				&line,
				//DrawMode::Stroke(StrokeMode::new(Color::BLACK, 10.0)),
				DrawMode::Fill(FillMode::color(Color::BLACK)),
				Transform {
					translation : Vec3 { x: x, y: y, z: 0.0 },
					..default()
				},
			));

			x += metrics.horiAdvance as f32 * coef;//100.;
		}

		//text_mesh	(&String::from(line), y, ass, commands);


		y			-= 100.;
	}
}
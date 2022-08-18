use bevy				:: prelude :: { * };
use bevy_text_mesh		:: prelude :: { * };
use bevy_fly_camera		:: { FlyCamera };
use bevy_infinite_grid	:: { InfiniteGridBundle };

use bevy::render::mesh::shape as render_shape;

use std :: io		:: { prelude :: * };
use std :: fs		:: { File };
use std :: path		:: { Path, PathBuf };

use ttf2mesh :: { TTFFile, Value, Glyph };

use super				:: { * };

pub fn camera(
	commands			: &mut Commands
) {
	let camera = commands.spawn_bundle(Camera3dBundle {
			transform: Transform {
				translation: Vec3::new(1.5, 3., 7.),
				..default()
			},
			..default()
		})
		.insert			(FlyCamera{ yaw : 0.0, pitch : 0.0, enabled_follow : false, max_speed : 0.07, ..default() })
		.insert_bundle	(PickingCameraBundle::default())
		.id				();

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
			..default()
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
	let min_dim		= 0.02;
	let max_dim		= 1.0;
	let min_color	= 0.1;
	let max_color	= 0.8;
	let offset		= 0.5;

	// X
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(max_dim, min_dim, min_dim))),
		material	: materials.add			(Color::rgb(max_color, min_color, min_color).into()),
		transform	: Transform::from_xyz	(offset, 0.0, 0.0),
		..Default::default()
	});
	// Y
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(min_dim, max_dim, min_dim))),
		material	: materials.add			(Color::rgb(min_color, max_color, min_color).into()),
		transform	: Transform::from_xyz	(0.0, offset, 0.0),
		..Default::default()
	});
	// Z
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(min_dim, min_dim, max_dim))),
		material	: materials.add			(Color::rgb(min_color, min_color, max_color).into()),
		transform	: Transform::from_xyz	(0.0, 0.0, offset),
		..Default::default()
	});
}

pub fn infinite_grid(
	commands		: &mut Commands,
) {
	commands.spawn_bundle(InfiniteGridBundle::default());
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

extern crate rustc_ast;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_parse;
extern crate rustc_lexer;

use rustc_session::parse::{ ParseSess} ;
use rustc_span::{ FileName, RealFileName };
use rustc_span::edition::Edition;
use rustc_span::Span;
use rustc_span::BytePos;
use rustc_lexer::TokenKind;
use rustc_ast::tokenstream::TokenTree;
use rustc_parse::lexer::nfc_normalize;

use std::io::{ Read };

pub fn text_mesh(
	text_in				: &String,
	pos					: Vec2,
	font_handle			: &Handle<TextMeshFont>,
	font_size			: SizeUnit,
	color				: Color,
	ass					: &Res<AssetServer>,
	commands			: &mut Commands,
) {
    commands.spawn_bundle(TextMeshBundle {
        text_mesh: TextMesh {
            text	: text_in.clone(),
            style	: TextMeshStyle {
                font     : font_handle.clone(),
                font_size: font_size,
                color    : color,
                ..Default::default()
            },
            size: TextMeshSize {
				wrapping : false,
                ..default()
            },
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(pos.x, pos.y, 0.),
            ..default()
        },
        ..default()
    })

	.insert_bundle(PickableBundle::default());
}

fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}

pub fn file_text(
	ass				: &Res<AssetServer>,
	commands		: &mut Commands
) {
	// let source_file_path	= Some(PathBuf::from("playground/test_tabs.rs"));
	// let source_file_path	= Some(PathBuf::from("playground/test_simple.rs"));
	let source_file_path = Some(PathBuf::from("playground/easy_spawn.rs"));
	// let source_file_path	= Some(PathBuf::from("playground/test_letter_spacing.rs"));
	let load_name 	= file_path_to_string(&source_file_path);
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

	// TODO: load fonts on startup or something to get it as asset here already
	let font_handle : Handle<TextMeshFont> = ass.load("fonts/droidsans-mono.ttf"); //("fonts/FiraMono-Medium.ttf");

	let font_file_path = Some(PathBuf::from("assets/fonts/droidsans-mono.ttf"));
	let mut font = ttf2mesh::TTFFile::from_file(font_file_path.unwrap()).unwrap();
	let mut reference_glyph : Glyph = font.glyph_from_char('a').unwrap();

	let font_size = 9.;
	let tab_size = 4; //.editorconfig

	rustc_span::create_session_if_not_set_then(Edition::Edition2021, |_| {
		let parser_session = ParseSess::with_silent_emitter(Some(String::from("FATAL MESSAGE AGGHHH")));

		let mut lines	= file_content.lines();
		let mut y		= 5.0;
		let mut column	= 0 as u32;
		let mut row		= 0 as u32;

		loop {
			let line_raw = match lines.next() {
				Some(l)	=> l,
				None	=> break,
			};
			let line = String::from(line_raw);
			let line_clone = line.clone();
			
			let mut cursor_lexer = rustc_lexer::tokenize(line_raw);
			let mut token_offset = 0;

			loop {
				let token_meta = cursor_lexer.next();
				if token_meta.is_none() {
					break;
				}
				let token = token_meta.unwrap();
				// println!("lexer {:?}", token);

				let token_start : usize = token_offset;
				let token_end : usize = token_offset + token.len as usize;
				let token_str = &line_raw[token_start..token_end];

				let mut color = Color::hex("bbbbbb").unwrap();

				// it's a word
				match token.kind {
					TokenKind::Ident => {
						let sym = nfc_normalize(token_str);
						let span = Span::with_root_ctxt(BytePos(token_start as u32), BytePos(token_end as u32));
						let token_kind_ast = rustc_ast::token::TokenKind::Ident(sym, false);
						let token_ast = rustc_ast::token::Token { kind: token_kind_ast, span: span };


						if token_ast.is_used_keyword() {
							color = Color::hex("e06c75").unwrap();
						}

						if token_ast.is_keyword(rustc_span::symbol::kw::Let) {
							color = Color::hex("56b6c2").unwrap();
						}

						if token_ast.is_bool_lit() {
							color = Color::hex("56b6c2").unwrap();
						}

						let chars : Vec<char> = token_str.chars().collect();
						if token.len > 1 && chars[0].is_uppercase() && !chars[1].is_uppercase() {
							color = Color::hex("61afef").unwrap();
						}

						if token.len > 1 && chars[0].is_uppercase() && chars[1].is_uppercase() {
							color = Color::hex("56b6c2").unwrap();
						}

						if token_ast.is_op() {
							println!("what is op? [{}]", token_str);
						}
					},
					TokenKind::OpenBrace | TokenKind::OpenBracket | TokenKind::OpenParen => {
						color = Color::hex("da70d6").unwrap();
					},
					TokenKind::CloseBrace | TokenKind::CloseBracket | TokenKind::CloseParen => {
						color = Color::hex("da70d6").unwrap();
					},
					TokenKind::LineComment { doc_style } => {
						color = Color::hex("676f7d").unwrap();
					},
					TokenKind::Literal { kind, suffix_start } => {
						let k = kind;
						match k {
							rustc_lexer::LiteralKind::Int { base: Base, empty_int: bool } => {
								color = Color::hex("c678dd").unwrap();
							},
							rustc_lexer::LiteralKind::Float { base: Base, empty_exponent: bool } => {
								color = Color::hex("c678dd").unwrap();
							},
							_ => {
								color = Color::hex("e5c07b").unwrap();
							},
						}
					}
					_ => {
						color = Color::hex("e06c75").unwrap();

						let sym = nfc_normalize(token_str);
						let nt = rustc_ast::token::NonterminalKind::from_symbol(sym, || -> Edition { Edition::Edition2021 });
					},
				}

				// println!("[{} {} {}] [{}]", column, token_offset, token.len, token_str);

				if token.kind != TokenKind::Whitespace {
					let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5
					let x		= (column as f32) * reference_glyph.inner.advance * font_size_scalar;
					// println!("x: {} column: {} advance: {} font_size: {}", x, column, reference_glyph.inner.advance, font_size);
					let pos		= Vec2::new(x, y);
					let mesh_string = String::from(token_str);

					text_mesh	(&mesh_string, pos, &font_handle, SizeUnit::NonStandard(font_size), color, ass, commands);
				}

				let mut len : u32 = token.len;
				
				if token_str.chars().next().unwrap() == '\t' {
					len = (token.len - 1) * tab_size; 

					let leftovers = tab_size - (column % tab_size);

					len += leftovers;
				}
				column += len;
				token_offset += token.len as usize; // amount of tokens != amount of symbols so we need to keep track of both 
			}

			// if row >= 9 {
			// 	break;
			// }

			y			-= 0.13;
			column		 = 0;
			row			+= 1;
		}

		
    });
}
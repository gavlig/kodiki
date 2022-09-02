use bevy				:: prelude :: { * };
use bevy_text_mesh		:: prelude :: { * };
// use bevy_infinite_grid	:: { InfiniteGridBundle };

use ttf2mesh 			:: { Glyph };

use super				:: { * };

extern crate rustc_ast;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_parse;
extern crate rustc_lexer;

use rustc_session		:: parse :: { ParseSess} ;
use rustc_span			:: edition :: Edition;
use rustc_lexer			:: TokenKind;

use mesh as spawn_mesh;

fn calc_vertical_offset(row : f32) -> f32 {
	row * -0.13
}

pub fn file(
	file_path		: &str,
	font_handle 	: &Handle<TextMeshFont>,
	font			: &mut ttf2mesh::TTFFile,
	world_position	: Vec3,
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands
) {
	let file_content = load_text_file(file_path).unwrap();

	let reference_glyph : Glyph = font.glyph_from_char('a').unwrap(); // and omega

	let local_position = Vec3::ZERO;
	let font_size	= 9.;
	let tab_size	= 4; //.editorconfig

	let mut children : Vec<Entity> = Vec::new();

	let header_string = format!("--[ {} ]--", file_path);
	children.push(
		spawn_mesh(
			&header_string,
			local_position + Vec3::Y * calc_vertical_offset(1.0),
			&font_handle,
			SizeUnit::NonStandard(font_size),
			Color::hex("bbbbbb").unwrap(),
			commands
		)
	);

	let mut lines	= file_content.lines();
	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 3 as u32;

	rustc_span::create_session_if_not_set_then(Edition::Edition2021, |_| {
		let _parser_session = ParseSess::with_silent_emitter(Some(String::from("FATAL MESSAGE AGGHHH")));

		loop {
			y		= calc_vertical_offset(row as f32);

			let line_raw = match lines.next() {
				Some(l)	=> l,
				None	=> break,
			};
			
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

				let color = color_from_token_kind(&token, token_str, token_start, token_end);
				// println!("[{} {} {}] [{}]", column, token_offset, token.len, token_str);

				// line/row numbers
				if column == 0 {
					let row_num_string = format!("{:>5} ", row);
					let pos		= local_position + (Vec3::Y * y);
					children.push(
						spawn_mesh(
							&row_num_string,
							pos,
							&font_handle,
							SizeUnit::NonStandard(font_size),
							Color::hex("495162").unwrap(),
							commands
						)
					);
				}

				// 
				if token.kind != TokenKind::Whitespace {
					let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5
					let row_num_offset = 6. * reference_glyph.inner.advance * font_size_scalar; 
					let column_offset = (column as f32) * reference_glyph.inner.advance * font_size_scalar;
					let x		= row_num_offset + column_offset;

					// println!("x: {} column: {} advance: {} font_size: {}", x, column, reference_glyph.inner.advance, font_size);

					let pos		= local_position + Vec3::new(x, y, 0.0);
					let mesh_string = String::from(token_str);

					children.push(
						spawn_mesh(
							&mesh_string,
							pos,
							&font_handle,
							SizeUnit::NonStandard(font_size),
							color,
							commands
						)
					);
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

			// Cheat/Debug
			// if row >= 9 {
			// 	break;
			// }

			column		 = 0;
			row			+= 1;
		}
	});

	let quad_width		= 10.0;
	let quad_height		= calc_vertical_offset(row as f32);
    let quad_handle		= meshes.add(
		Mesh::from(
			shape::Quad::new(
				Vec2::new(
					quad_width,
					quad_height
    			)
			)
		)
	);
	let quad_position	= Vec3::new(quad_width / 2.0, quad_height / 2.0, 0.0);

    let blue_material_handle = materials.add(StandardMaterial {
        base_color		: Color::hex("282c34").unwrap(),
        // alpha_mode	: AlphaMode::Opaque,
        unlit			: true,
        // double_sided	: true,
        ..default()
    });

	children.push(
		commands.spawn_bundle(PbrBundle {
			mesh			: quad_handle,
			material		: blue_material_handle,
			transform		: Transform {
				translation	: quad_position,
				rotation	: Quat::from_rotation_y(std::f32::consts::PI),
				..default()
			},
			..default()
		})
		.id()
	);

	//
	//
	//

	let root_entity =
	commands.spawn_bundle(TransformBundle {
		local			: Transform::from_translation(world_position),
		..default()
	})
	.insert_bundle(VisibilityBundle {
		visibility: Visibility { is_visible: true },
		..default()
	})
	.id();

	commands.entity(root_entity).push_children(children.as_slice());
}

pub fn mesh(
	text_in				: &String,
	pos					: Vec3,
	font_handle			: &Handle<TextMeshFont>,
	font_size			: SizeUnit,
	color				: Color,
	commands			: &mut Commands,
) -> Entity {
    commands.spawn_bundle(TextMeshBundle {
        text_mesh: TextMesh {
            text		: text_in.clone(),
            style		: TextMeshStyle {
                font     : font_handle.clone(),
                font_size: font_size,
                color    : color,
                ..default()
            },
            size: TextMeshSize {
				wrapping : false,
                ..default()
            },
            ..default()
        },
        transform: Transform {
            translation: pos,
            ..default()
        },
        ..default()
    })
	// .insert_bundle(PickableBundle::default())
	.id()
}
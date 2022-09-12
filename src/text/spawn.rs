use bevy				:: prelude :: { * };
use bevy_text_mesh		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_fly_camera		:: { * };

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use ttf2mesh 			:: { Glyph };

use super				:: { * };

extern crate rustc_ast;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_parse;
extern crate rustc_lexer;

use rustc_session		:: parse :: { ParseSess };
use rustc_span			:: edition :: Edition;
use rustc_lexer			:: TokenKind;

use mesh as spawn_mesh;

fn calc_vertical_offset(row : f32, reference_glyph : &Glyph) -> f32 {
	row * -0.13 // (reference_glyph.inner.ybounds[0] - reference_glyph.inner.ybounds[1]) / 72.
}

fn quad(
	quad_pos_in		: Vec3,
	quad_size		: Vec2,
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands
) -> Entity {
	let quad_width		= quad_size.x;
	let quad_height		= quad_size.y;

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
	let quad_pos		= quad_pos_in + Vec3::new(quad_width / 2.0, 0., 0.);//-quad_height / 2.0, 0.0);

    let blue_material_handle = materials.add(StandardMaterial {
        base_color		: Color::hex("282c34").unwrap(),
        // alpha_mode	: AlphaMode::Opaque,
        unlit			: true,
        // double_sided	: true,
        ..default()
    });

	commands.spawn_bundle(PbrBundle {
		mesh			: quad_handle,
		material		: blue_material_handle,
		transform		: Transform {
			translation	: quad_pos,
			// rotation	: Quat::from_rotation_y(std::f32::consts::PI), // winding ccw something something
			..default()
		},
		..default()
	})
	.insert(PickableMesh::default())
	.id()
}

const DEFAULT_FONT_SIZE  : f32 = 36.;
const DEFAULT_FONT_WIDTH : f32 = DEFAULT_FONT_SIZE * 10.;
const DEFAULT_FONT_HEIGHT: f32 = DEFAULT_FONT_SIZE * 5.;
const DEFAULT_FONT_DEPTH : f32 = DEFAULT_FONT_SIZE * 0.10;

pub fn mesh(
	text_in				: &String,
	pos					: Vec3,
	font_handle			: &Handle<TextMeshFont>,
	font_size			: SizeUnit,
	font_depth			: f32,
	color				: Color,
	commands			: &mut Commands,
) -> Entity {
    commands.spawn_bundle(TextMeshBundle {
        text_mesh: TextMesh {
            text		: text_in.clone(),
            style		: TextMeshStyle {
				color	: color,
                font     : font_handle.clone(),
                font_size : font_size,
                ..default()
            },
            size: TextMeshSize {
				depth	: Some(SizeUnit::NonStandard(DEFAULT_FONT_SIZE * font_depth)),
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

pub fn file(
	file_path		: &str,
	font_handle 	: &Handle<TextMeshFont>,
	font			: &mut ttf2mesh::TTFFile,
	world_position	: Vec3,
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands
) -> Entity
{
	let file_content = load_text_file(file_path).unwrap();

	let tab_size	= 4; //.editorconfig
	let font_size	= 9.;
	let font_depth	= 0.1;
	let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5

	let reference_glyph : Glyph = font.glyph_from_char('a').unwrap(); // and omega
	let row_offset = calc_vertical_offset(1.0, &reference_glyph);
	let glyph_width	= reference_glyph.inner.advance * font_size_scalar;
	let glyph_height = row_offset.abs();
	let row_num_offset = 6. * glyph_width;
	let vertical_overlap = 0.05;

	let local_position = Vec3::ZERO;

	let mut children : Vec<Entity> = Vec::new();

	let header_string = format!("-- [ {} ] --", file_path);
	children.push(
		spawn_mesh(
			&header_string,
			local_position + Vec3::new(row_num_offset, row_offset, 0.0),
			&font_handle,
			SizeUnit::NonStandard(font_size),
			font_depth,
			Color::hex("bbbbbb").unwrap(),
			commands
		)
	);
	

	let mut lines	= file_content.lines();
	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 3 as u32;
	let mut empty_line = false;
	let mut column_max = 0 as u32;
	let mut row_max = 0 as u32;

	rustc_span::create_session_if_not_set_then(Edition::Edition2021, |_| {
		let _parser_session = ParseSess::with_silent_emitter(Some(String::from("FATAL MESSAGE AGGHHH")));

		loop {
			y = calc_vertical_offset(row as f32, &reference_glyph);

			// mesh for line/row numbers
			let row_num_string = format!("{:>5} ", row);
			let pos		= local_position + (Vec3::Y * y);
			children.push(
				spawn_mesh(
					&row_num_string,
					pos,
					&font_handle,
					SizeUnit::NonStandard(font_size),
					font_depth,
					Color::hex("495162").unwrap(),
					commands
				)
			);

			// get next line of characters
			let line_raw = match lines.next() {
				Some(l)	=> l,
				None	=> break,
			};
			
			let mut cursor_lexer = rustc_lexer::tokenize(line_raw);

			let mut token_offset = 0;
			let was_empty_line = empty_line;

			loop {
				let token_meta = cursor_lexer.next();
				if token_meta.is_none() {
					if column == 0 {
						empty_line = true;
					}

					break;
				} else {
					empty_line = false;
				}
				let token = token_meta.unwrap();
				// println!("{}/{} lexer {:?}", row, column, token);

				let token_start : usize = token_offset;
				let token_end : usize = token_offset + token.len as usize;
				let token_str = &line_raw[token_start..token_end];

				let color = color_from_token_kind(&token, token_str, token_start, token_end);
				// println!("[{} {} {}] [{}]", column, token_offset, token.len, token_str);

				{
					let column_offset = (column as f32) * glyph_width;
					let x = row_num_offset + column_offset;

					let pos = local_position + Vec3::new(x, y, 0.0);
					let mesh_string = String::from(token_str);

					let mesh_entity_id =
					spawn_mesh(
						&mesh_string,
						pos,
						&font_handle,
						SizeUnit::NonStandard(font_size),
						font_depth,
						color,
						commands
					);
					children.push(mesh_entity_id);
				}

				// make a mesh for each character in this token
				for c in token_str.chars() {
					let column_offset = (column as f32) * glyph_width;
					let x = row_num_offset + column_offset;

					// let pos = local_position + Vec3::new(x, y, 0.0);
					// let mesh_string = String::from(c);

					let mut token_len = 1;
					if token.kind != TokenKind::Whitespace {
						// let mesh_entity_id =
						// spawn_mesh(
						// 	&mesh_string,
						// 	pos,
						// 	&font_handle,
						// 	SizeUnit::NonStandard(font_size),
						// 	font_depth,
						// 	color,
						// 	commands
						// );
						// children.push(mesh_entity_id);
					} else if c == '\t' {
						token_len = tab_size - (column % tab_size);
					}

					column += token_len;
					
					// background quad
					// let quad_width		= glyph_width * token_len as f32;
					// let quad_height		= glyph_height;
					// let quad_pos		= Vec3::new(x, y, -0.25 / 72.);
					// let quad_entity_id	= 
					// quad(
					// 	quad_pos,
					// 	Vec2::new(quad_width, quad_height),
					// 	meshes,
					// 	materials,
					// 	commands
					// );

					// commands.entity(quad_entity_id)
					// .insert(Row { 0: row })
					// .insert(Column { 0: column })
					// ;

					// children.push(quad_entity_id);
				}

				token_offset += token.len as usize; // amount of tokens != amount of symbols so we need to keep track of both 
			}

			if was_empty_line {
				// background quad for previous line
				// let quad_width		= glyph_width * column as f32;
				// let quad_height		= glyph_height;
				// let y 				= calc_vertical_offset((row - 1) as f32, &reference_glyph);
				// let quad_pos		= Vec3::new(row_num_offset, y, -0.25 / 72.);
				// let quad_entity_id	= 
				// quad(
				// 	quad_pos,
				// 	Vec2::new(quad_width, quad_height),
				// 	meshes,
				// 	materials,
				// 	commands
				// );

				// commands.entity(quad_entity_id)
				// .insert(Row { 0: row })
				// .insert(Column { 0: column })
				// ;
			}

			// Cheat/Debug
			// if row >= 9 {
			// 	break;
			// }

			column_max	= column_max.max(column);

			column		 = 0;
			row			+= 1;
		}
	});

	row_max				= row;

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

	commands.entity(root_entity)
	.insert(ReaderData {
		rows: row_max,
		columns: column_max,
		glyph_width: glyph_width,
		glyph_height: glyph_height
	})
	.push_children(children.as_slice());

	root_entity
}
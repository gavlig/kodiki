use bevy				:: prelude :: { * };
use bevy_text_mesh		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_fly_camera		:: { * };
use bevy_contrib_colors	:: { Tailwind };

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use ttf2mesh 			:: { Glyph };

use super				:: { * };

use mesh as spawn_mesh;
use bevy::render::mesh::shape as render_shape;

use helix_tui 			:: { buffer :: Buffer as Surface };
use helix_view::graphics::Color as HelixColor;

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

pub fn helix_surface(
	surface			: &Surface,
	file_path		: &str,
	font_handle 	: &Handle<TextMeshFont>,
	font			: &mut ttf2mesh::TTFFile,
	world_position	: Vec3,
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands
) -> (Entity, TextDescriptor)
{
	// let file_content = load_text_file(file_path).unwrap();

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

	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 3 as u32;
	let mut empty_line = false;
	let mut column_max = 0 as u32;
	let mut row_max = 0 as u32;
	
	let width = surface.area.width;
	let height = surface.area.height;
	let content = surface.content;

	for y_cell in 0..height {
		y = calc_vertical_offset(row as f32, &reference_glyph);
		
		for x_cell in 0..width {
			let cell = content[y_cell * width + x_cell];

			let column_offset = (column as f32) * glyph_width;
			let x = row_num_offset + column_offset;

			let pos = local_position + Vec3::new(x, y, 0.0);

			let color = match cell.fg {
				HelixColor::Reset => Color::White,
				HelixColor::Black => CColor::Black,
				HelixColor::Red => CColor::DarkRed,
				HelixColor::Green => CColor::DarkGreen,
				HelixColor::Yellow => CColor::DarkYellow,
				HelixColor::Blue => CColor::DarkBlue,
				HelixColor::Magenta => CColor::DarkMagenta,
				HelixColor::Cyan => CColor::DarkCyan,
				HelixColor::Gray => CColor::DarkGrey,
				HelixColor::LightRed => CColor::Red,
				HelixColor::LightGreen => CColor::Green,
				HelixColor::LightBlue => CColor::Blue,
				HelixColor::LightYellow => CColor::Yellow,
				HelixColor::LightMagenta => CColor::Magenta,
				HelixColor::LightCyan => CColor::Cyan,
				HelixColor::LightGray => CColor::Grey,
				HelixColor::White => CColor::White,
				HelixColor::Indexed(i) => CColor::AnsiValue(i),
				HelixColor::Rgb(r, g, b) => CColor::Rgb { r, g, b },
			}

			let mut token_len = 1;
			// if token.kind != TokenKind::Whitespace {
				let mesh_entity_id =
				spawn_mesh(
					&cell.symbol,
					pos,
					&font_handle,
					SizeUnit::NonStandard(font_size),
					font_depth,
					color,
					commands
				);
				children.push(mesh_entity_id);
			// } else if c == '\t' {
				// token_len = tab_size - (column % tab_size);
			// }

			column += token_len;
		}

		column_max	= column_max.max(column);

		column		 = 0;
		row			+= 1;
	}

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

	let text_descriptor = TextDescriptor {
		rows: row_max,
		columns: column_max,
		glyph_width: glyph_width,
		glyph_height: glyph_height
	};

	commands.entity(root_entity)
	.insert(text_descriptor.clone())
	.push_children(children.as_slice());

	(root_entity, text_descriptor)
}

pub fn caret(
	parent_entity	: Entity,	
	text_descriptor	: &TextDescriptor,
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands
) {
	let min_dim		= text_descriptor.glyph_width / 2.0;
	let max_dim		= text_descriptor.glyph_height;

	let transform	= Transform::identity();

	let caret_entity =
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add	(Mesh::from(render_shape::Box::new(min_dim, max_dim, min_dim))),
		material	: materials.add	(Tailwind::GRAY100.into()),
		transform	: transform,
		..default()
	})
	.insert(Caret::default())
	.id()
	;

	commands.entity(parent_entity)
	.add_child(caret_entity)
	;
}
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

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
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
	.id()
}

fn color_from_helix(helix_color: HelixColor) -> Color {
	match helix_color {
		HelixColor::Reset => Color::WHITE,
		HelixColor::Black => Color::BLACK,
		HelixColor::Red => Tailwind::RED600,
		HelixColor::Green => Tailwind::GREEN600,
		HelixColor::Yellow => Tailwind::YELLOW600,
		HelixColor::Blue => Tailwind::BLUE600,
		HelixColor::Magenta => Tailwind::PURPLE600,
		HelixColor::Cyan => Color::rgb(0.0, 0.5, 0.5),
		HelixColor::Gray => Tailwind::GRAY600,
		HelixColor::LightRed => Tailwind::RED300,
		HelixColor::LightGreen => Tailwind::GREEN300,
		HelixColor::LightBlue => Tailwind::BLUE300,
		HelixColor::LightYellow => Tailwind::YELLOW300,
		HelixColor::LightMagenta => Tailwind::PURPLE300,
		HelixColor::LightCyan => Color::rgb(0.0, 0.7, 0.7),
		HelixColor::LightGray => Tailwind::GRAY300,
		HelixColor::White => Color::WHITE,
		// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
		HelixColor::Indexed(_i) => { panic!("Indexed color is not supported!"); },// Color::AnsiValue(i), 
		HelixColor::Rgb(r, g, b) => Color::rgb_u8(r, g, b),
	}
}

pub fn surface(
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,
	font_handle 	: &Handle<TextMeshFont>,
	font			: &mut ttf2mesh::TTFFile,
	world_position	: Vec3,
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands
) -> (Entity, TextDescriptor)
{
	surface_bevy.content.resize_with(surface_helix.content.len(), || { CellBevy::default() });

	let font_size	= 9.;
	let font_depth	= 0.1;
	let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5

	let reference_glyph : Glyph = font.glyph_from_char('a').unwrap(); // and omega
	let row_offset = calc_vertical_offset(1.0, &reference_glyph);
	let glyph_width	= reference_glyph.inner.advance * font_size_scalar;
	let glyph_height = row_offset.abs();

	let local_position = Vec3::ZERO;

	let mut children : Vec<Entity> = Vec::new();

	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 0 as u32;
	// let mut empty_line = false;
	let mut column_max = 0 as u32;
	let mut row_max = 0 as u32;
	
	let width = surface_helix.area.width;
	let height = surface_helix.area.height;
	let content_helix = &surface_helix.content;
	let content_bevy = &mut surface_bevy.content;

	for y_cell in 0..height {
		y = calc_vertical_offset(row as f32, &reference_glyph);
		
		for x_cell in 0..width {
			let content_index = (y_cell * width + x_cell) as usize;
			let _cell_helix = &content_helix[content_index];
			let _cell_bevy = &mut content_bevy[content_index];

			// println!("[{} {}] cell {}", x_cell, y_cell, cell.symbol);

			let column_offset = (column as f32) * glyph_width;
			let x = column_offset;

			let pos = local_position + Vec3::new(x, y, 0.0);

			// use crate :: game :: spawn :: WorldAxisDesc;
			// use crate :: game :: spawn :: world_axis as spawn_world_axis;

			// spawn_world_axis(
			// 	Transform::from_translation(pos),
			// 	WorldAxisDesc {
			// 		min_dim : 0.005,
			// 		max_dim : 0.04,
			// 		offset	: 0.001,
			// 	},
			// 	meshes,
			// 	materials,
			// 	commands
			// );


			column += 1;
		}

		column_max	= column_max.max(column);

		column		= 0;
		row			+= 1;
	}

	row_max			= row;

	//
	//
	//

	let root_entity =
	commands.spawn_bundle(TransformBundle {
		local			: Transform::from_translation(world_position),
		..default()
	})
	.insert_bundle(VisibilityBundle {
		visibility		: Visibility { is_visible: true },
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
	.insert(BevyHelix)
	.push_children(children.as_slice());

	(root_entity, text_descriptor)
}
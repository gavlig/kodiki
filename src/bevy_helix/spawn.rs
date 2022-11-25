use bevy				:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_fly_camera		:: { * };
use bevy_contrib_colors	:: { Tailwind };

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use super				:: { * };

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
use helix_view :: graphics :: Color as HelixColor;

use crate :: bevy_ab_glyph :: ABGlyphFont;
use crate :: bevy_ab_glyph :: TextMeshesCache;

fn quad_old(
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

fn quad(
	quad_pos_in		: Vec3,
	quad_size		: Vec2,
	quad_mesh_handle: Handle<Mesh>,
	commands		: &mut Commands
) -> Entity {
	let quad_width		= quad_size.x;
	let quad_height		= quad_size.y;

	let quad_pos		= quad_pos_in + Vec3::new(quad_width / 2.0, 0., 0.);//-quad_height / 2.0, 0.0);

	commands.spawn_bundle(PbrBundle {
		mesh			: quad_mesh_handle.clone_weak(),
		transform		: Transform {
			translation	: quad_pos,
			// rotation	: Quat::from_rotation_y(std::f32::consts::PI), // winding ccw something something
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
	name			: &String,
	world_position	: Vec3,

	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,

	font			: &ABGlyphFont,

	text_mesh_cache	: &mut TextMeshesCache,
	helix_colors_cache : &mut HelixColorsCache,

	mesh_assets		: &mut Assets<Mesh>,
	material_assets : &mut Assets<StandardMaterial>,
	commands		: &mut Commands
) -> Entity
{
	surface_bevy.content.resize_with(surface_helix.content.len(), || { CellBevy::default() });

	let v_advance	= font.vertical_advance(font.scale);
	let h_advance	= font.scale; // the widest glyph is supposed to be 1.0 so just scale should be enough
	
	let mut children : Vec<Entity> = Vec::new();

	// let mut y		= 0.0;
	// let mut column	= 0 as u32;
	// let mut row		= 0 as u32;
	
	let width		= surface_helix.area.width;
	let height		= surface_helix.area.height;
	// let content_helix = &surface_helix.content;
	// let content_bevy = &mut surface_bevy.content;
	
	// println!("spawn surface {} len {} w {} h {}", name, surface_helix.content.len(), width, height);
	
	// for y_cell in 0..height {
	// 	let y 		= calc_vertical_offset(y_cell as f32) + (v_advance / 2.0) + (ybounds[0] * font_size_scalar);
		
	// 	for x_cell in 0..width {
	// 		let content_index = (y_cell * width + x_cell) as usize;
	// 		let cell_helix	= &content_helix[content_index];
	// 		let cell_bevy	= &mut content_bevy[content_index];

	// 		let column_offset = (x_cell as f32) * h_advance;
	// 		let x 			= column_offset + (h_advance / 2.0) + lbearing;
	// 		let pos = Vec3::new(x, y, 0.0);
			
	// 		let quad_width	= h_advance;
	// 		let quad_height	= v_advance;
			
	// 		//
	// 		//
	// 		// Background Quad
			
	// 		// mesh handle
	// 		let quad_mesh_name = String::from("character-background-quad");
	// 		let quad_mesh_handle = match text_mesh_cache.meshes.get(&quad_mesh_name) {
	// 			Some(handle) => handle.clone(),
	// 			None => {
	// 				let handle = mesh_assets.add(
	// 					Mesh::from(
	// 						shape::Quad::new(
	// 							Vec2::new(
	// 								quad_width,
	// 								quad_height
	// 			    			)
	// 						)
	// 					)
	// 				);
					
	// 				text_mesh_cache.meshes.insert_unique_unchecked(quad_mesh_name.clone(), handle).1.clone()
	// 			}
	// 		};
			
	// 		let quad_pos		= Vec3::new(x, y, -0.25 / 72.);
	// 		let quad_entity_id	= 
	// 		quad(
	// 		 	quad_pos,
	// 		 	Vec2::new(quad_width, quad_height),
	// 		 	quad_mesh_handle,
	// 		 	commands
	// 		);
			
	// 		cell_bevy.entity_bg_quad = Some(quad_entity_id);

	// 		commands.entity(quad_entity_id)
	// 		.insert(Row { 0: row })
	// 		.insert(Column { 0: column })
	// 		;

	// 		children.push(quad_entity_id);

	// 		column 	+= 1;
	// 	}

	// 	column		= 0;
	// 	row			+= 1;
	// }

	// //
	// //
	// //

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
		rows		: height as u32,
		columns		: width as u32,
		glyph_width	: h_advance,
		glyph_height: v_advance
	};

	commands.entity(root_entity)
		.insert(text_descriptor)
		.insert(BevyHelix)
		;
	
	if children.len() > 0 {
		commands.entity(root_entity).push_children(children.as_slice());
	}

	root_entity
}
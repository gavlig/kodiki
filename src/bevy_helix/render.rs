use bevy				:: prelude :: { * };
use bevy_debug_text_overlay::screen_print;
use bevy_text_mesh		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_fly_camera		:: { * };
use bevy_contrib_colors	:: { Tailwind };

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use ttf2mesh 			:: { Glyph };

use super				:: { * };
use crate				:: game :: DespawnResource;

use helix_tui 			:: { buffer :: Buffer as SurfaceHelix };
use helix_tui			:: { buffer :: Cell as CellHelix };
use helix_view::graphics::Color as HelixColor;

fn calc_vertical_offset(row : f32) -> f32 {
	row * -0.13 // (reference_glyph.inner.ybounds[0] - reference_glyph.inner.ybounds[1]) / 72.
}

fn quad(
	quad_pos_in		: Vec3,
	quad_size		: Vec2,
	meshes			: &mut ResMut<Assets<Mesh>>,
	material_handle	: &Handle<StandardMaterial>,
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

	commands.spawn_bundle(PbrBundle {
		mesh			: quad_handle,
		material		: material_handle.clone(),
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

const DEFAULT_FONT_SIZE  : f32 = 36.;
const DEFAULT_FONT_WIDTH : f32 = DEFAULT_FONT_SIZE * 10.;
const DEFAULT_FONT_HEIGHT: f32 = DEFAULT_FONT_SIZE * 5.;
const DEFAULT_FONT_DEPTH : f32 = DEFAULT_FONT_SIZE * 0.10;

fn mesh_from_symbol(
	text_in				: &String,
	font				: &mut TTFFile,
	font_size			: SizeUnit,
	font_depth			: f32,
	ttf2_mesh_cache		: &mut TTF2MeshCache,
	meshes				: &mut Assets<Mesh>,
) -> Handle<Mesh> {
	let text_mesh_desc	= TextMesh {
		text			: text_in.clone(),
		style			: TextMeshStyle {
			font_size,
			..default()
		},
		size: TextMeshSize {
			depth	: Some(SizeUnit::NonStandard(DEFAULT_FONT_SIZE * font_depth)),
			wrapping : false,
			..default()
		},
		
		..default()
	};
		
	generate_text_mesh(&text_mesh_desc, font, meshes, Some(ttf2_mesh_cache))
}

pub fn surface(
	offset			: Vec3,
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,
	font			: &mut ttf2mesh::TTFFile,
	ttf2_mesh_cache	: &mut TTF2MeshCache,
	mesh_cache		: &mut MeshesMap,
	helix_colors_cache : &mut MaterialsMap,
	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	_despawn		: &mut DespawnResource,
	commands		: &mut Commands
)
{
	let root_entity = surface_bevy.entity.unwrap();

	surface_bevy.content.resize_with(surface_helix.content.len(), || { CellBevy::default() });

	let font_size	= 9.;
	let font_depth	= 0.1;
	let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5

	let reference_glyph : Glyph = font.glyph_from_char('a').unwrap(); // and omega
	let row_offset = calc_vertical_offset(1.0);
	let glyph_width	= reference_glyph.inner.advance * font_size_scalar;
	let glyph_height = row_offset.abs();

	let mut children : Vec<Entity> = Vec::new();

	let mut y		= 0.0;
	let mut column	= 0 as u32;
	let mut row		= 0 as u32;
	
	let width = surface_helix.area.width;
	let height = surface_helix.area.height;
	let content_helix = &surface_helix.content;
	let content_bevy = &mut surface_bevy.content;

	for y_cell in 0..height {
		y = calc_vertical_offset(row as f32);
		
		for x_cell in 0..width {
			let content_index = (y_cell * width + x_cell) as usize;
			let cell_helix = &content_helix[content_index];
			let cell_bevy = &mut content_bevy[content_index];
			
			//
			//
			// Background
			

			//
			//
			// Character
			
			// figure out colors first
			let reversed = cell_helix.modifier == helix_view::graphics::Modifier::REVERSED;
			let wrong_color = !reversed && (cell_bevy.fg != cell_helix.fg || cell_bevy.bg != cell_helix.bg);
			let reversed_and_wrong_color = reversed && (cell_bevy.fg != cell_helix.bg || cell_bevy.bg != cell_helix.fg);

			if wrong_color || reversed_and_wrong_color {
				update_cell_materials(
					cell_bevy,
					cell_helix,
					reversed,
					helix_colors_cache,
					material_assets,
					commands
				);
			}

			// now spawn new mesh if needed
			let x = (column as f32) * glyph_width;
			let pos = offset + Vec3::new(x, y, 0.0);

			let wrong_symbol = cell_helix.symbol != cell_bevy.symbol;
			if wrong_symbol {
				// println!("[{} {}] wrong symbol [{}] <= [{}]", x_cell, y_cell, cell_helix.symbol, cell_bevy.symbol);

				update_cell_mesh(
					pos,
					cell_helix,
					cell_bevy,
					font,
					font_size,
					font_depth,
					mesh_cache,
					ttf2_mesh_cache,
					&mut children,
					mesh_assets,
					commands
				);

				// print!("\n");
			}

			column += 1;
		}

		column		= 0;
		row			+= 1;
	}

	if children.len() > 0 {
		commands.entity(root_entity).push_children(children.as_slice());
	}
}

fn update_cell_mesh(
	pos				: Vec3,
	cell_helix		: &CellHelix,
	cell_bevy		: &mut CellBevy,
	font			: &mut TTFFile,
	font_size		: f32, 
	font_depth		: f32,
	mesh_cache		: &mut MeshesMap,
	ttf2_mesh_cache	: &mut TTF2MeshCache,
	children		: &mut Vec<Entity>,
	meshes			: &mut Assets<Mesh>,
	commands		: &mut Commands
) {
    let cache = mesh_cache.get(&cell_helix.symbol);
    let cache_found = cache.is_some();
    let space_symbol = cell_helix.symbol == " ";
    if !cache_found && !space_symbol {
		// println!("cache not found for [{}]", cell_helix.symbol);

		if cell_bevy.entity_symbol.is_none() {
			// println!("spawning new entity for [{}] pos: {:?} ", cell_helix.symbol, pos);

			cell_bevy.entity_symbol = Some(
				commands.spawn_bundle(
					PbrBundle {
						transform : Transform::from_translation(pos),
						..default()
					}
				)
				.id()
			);
		}
		let mesh_entity_id = cell_bevy.entity_symbol.unwrap();

		let mesh_handle =
		mesh_from_symbol(
			&cell_helix.symbol,
			font,
			SizeUnit::NonStandard(font_size),
			font_depth,
			ttf2_mesh_cache,
			meshes
		);

		// insert mesh
		commands.entity(mesh_entity_id).insert(mesh_handle.clone_weak());

		// insert material
		commands.entity(mesh_entity_id).insert(
			cell_bevy.fg_handle.as_ref().unwrap().clone_weak()
		);

		children.push(mesh_entity_id);
		mesh_cache.insert(cell_helix.symbol.clone(), mesh_handle);

		cell_bevy.entity_symbol = Some(mesh_entity_id);
	} else if !cache_found && space_symbol {
		// println!("cache not found but we dont care it's space");

		if let Some(entity) = cell_bevy.entity_symbol {
			// remove mesh
			commands.entity(entity).remove::<Handle<Mesh>>();
		}
	} else if let Some(cache) = cache {
		// println!("cache found for [{}]", cell_helix.symbol);

		if let Some(entity) = cell_bevy.entity_symbol {
			// println!("replacing mesh handle for [{}]", cell_helix.symbol);

			// replace previous mesh with new one
			commands.entity(entity)
				.remove::<Handle<Mesh>>()
				.insert(cache.clone_weak())
				;
		} else {
			// println!("spawning new entity with an existing mesh for [{}] pos: {:?}", cell_helix.symbol, pos);

			// spawn new entity with an existing mesh
			cell_bevy.entity_symbol = Some(
				commands.spawn_bundle(PbrBundle {
					mesh : cache.clone_weak(),
					material : cell_bevy.fg_handle.as_ref().unwrap().clone_weak(),
					transform : Transform::from_translation(pos),
					..default()
				})
				.id()
			);
			children.push(cell_bevy.entity_symbol.unwrap());
		}
	}
    cell_bevy.symbol = cell_helix.symbol.clone();
}

pub fn update_cell_materials(
	cell_bevy: &mut CellBevy,
	cell_helix: &CellHelix,
	reversed: bool,
	helix_colors_cache: &mut MaterialsMap,
	material_assets: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	// first take care of reversed colors: if reversed foreground becomes background
    (cell_bevy.fg, cell_bevy.bg) =
	if !reversed {
		(cell_helix.fg, cell_helix.bg)
	} else {
		(cell_helix.bg, cell_helix.fg)
	};

    let color_fg = color_from_helix(cell_bevy.fg);
    let color_bg = color_from_helix(cell_bevy.bg);

    cell_bevy.fg_handle = Some(get_helix_color_material_handle(color_fg, helix_colors_cache, material_assets));
    cell_bevy.bg_handle = Some(get_helix_color_material_handle(color_bg, helix_colors_cache, material_assets));

    // replace material to reflect changed color
    if let Some(cell_bevy_entity_symbol) = cell_bevy.entity_symbol {
		commands.entity		(cell_bevy_entity_symbol)
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.fg_handle.as_ref().unwrap().clone())
		;
	}
	
	if let Some(cell_bevy_entity_bg_quad) = cell_bevy.entity_bg_quad {
		commands.entity		(cell_bevy_entity_bg_quad)
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.bg_handle.as_ref().unwrap().clone())
		;
	}
}

pub fn cursor(
	offset			: Vec3,
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &SurfaceBevy,
	font			: &mut ttf2mesh::TTFFile,
	cursor			: &mut CursorBevy,
	q_cursor_transform : &mut Query<&mut Transform>,
	time			: &Res<Time>,
	meshes			: &mut ResMut<Assets<Mesh>>,
	commands		: &mut Commands
)
{
	let root_entity = surface_bevy.entity.unwrap();

	let font_size	= 9.;
	let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5

	let ybounds = {
		let reference_glyph_y : Glyph = font.glyph_from_char('y').unwrap();
		reference_glyph_y.inner.ybounds
	};

	let reference_glyph : Glyph = font.glyph_from_char('a').unwrap(); // and omega
	
	let row_offset			= calc_vertical_offset(1.0);
	let lbearing			= reference_glyph.inner.lbearing * font_size_scalar;
	let glyph_width			= reference_glyph.inner.advance * font_size_scalar;
	let glyph_height		= row_offset.abs();

	let mut children : Vec<Entity> = Vec::new();

	let width				= surface_helix.area.width;
	let content_helix 		= &surface_helix.content;
	let content_bevy 		= &surface_bevy.content;

	// move background quad
	if cursor.entity.is_some() && cursor.easing_accum < 1.0 {
		let cursor_entity 	= cursor.entity.unwrap();
		let column_offset 	= (cursor.x as f32) * glyph_width;
		let target_x 		= column_offset + (glyph_width / 2.0) + lbearing;
		let target_y 		= calc_vertical_offset(cursor.y as f32) + (glyph_height / 2.0) + (ybounds[0] * font_size_scalar);

		let target_pos		= offset + Vec3::new(target_x, target_y, -0.25 / 72.);

		let delta_seconds	= time.delta_seconds();
		let delta_accum		= delta_seconds / /*cursor_easing_seconds*/0.05;

		cursor.easing_accum = (cursor.easing_accum + delta_accum).min(1.0);
		let mut cursor_transform = q_cursor_transform.get_mut(cursor_entity).unwrap();
		cursor_transform.translation = cursor_transform.translation.lerp(target_pos, cursor.easing_accum);
	}

	let content_index 		= (cursor.y * width + cursor.x) as usize;
	let cell_helix			= &content_helix[content_index];
	let cell_bevy			= &content_bevy[content_index];

	let color_bg 			= color_from_helix(cell_bevy.bg);

	// spawn background quad for cursor
	if cursor.entity == None && cell_bevy.bg_handle.is_some() {
		let quad_width		= glyph_width;
		let quad_height		= (ybounds[1] - ybounds[0]) * font_size_scalar * 1.7; // ybounds contain offset for letter 'y'
		let quad_pos		= Vec3::new(0., 0., -0.25 / 72.);

		let quad_entity_id	= 
		quad(
			quad_pos,
			Vec2::new		(quad_width, quad_height),
			meshes,
			cell_bevy.bg_handle.as_ref().unwrap(),
			commands
		);

		children.push		(quad_entity_id);

		cursor.entity 		= Some(quad_entity_id);
		cursor.color		= color_bg;
	} else if cursor.color != color_bg && cell_bevy.bg_handle.is_some() {
		commands.entity		(cursor.entity.unwrap())
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.bg_handle.as_ref().unwrap().clone())
		;
		cursor.color		= color_bg;
	}

	if children.len() > 0 {
		commands.entity(root_entity).push_children(children.as_slice());
	}
}
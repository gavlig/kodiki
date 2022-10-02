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
			font_size 	: font_size,
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
	root_entity		: Entity,
	surface_helix	: &SurfaceHelix,
	surface_bevy	: &mut SurfaceBevy,
	font			: &mut ttf2mesh::TTFFile,
	ttf2_mesh_cache	: &mut TTF2MeshCache,
	text_mesh_cache	: &mut MeshesMap,
	meshes			: &mut Assets<Mesh>,
	materials		: &mut Assets<StandardMaterial>,
	_despawn			: &mut DespawnResource,
	commands		: &mut Commands
)
{
	surface_bevy.content.resize_with(surface_helix.content.len(), || { CellBevy::default() });

	let font_size	= 9.;
	let font_depth	= 0.1;
	let font_size_scalar = font_size / 72.; // see SizeUnit::as_scalar5

	let reference_glyph : Glyph = font.glyph_from_char('a').unwrap(); // and omega
	let row_offset = calc_vertical_offset(1.0);
	let glyph_width	= reference_glyph.inner.advance * font_size_scalar;
	let glyph_height = row_offset.abs();

	let local_position = Vec3::ZERO;

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

			// figure out colors first
			let reversed = cell_helix.modifier == helix_view::graphics::Modifier::REVERSED;
			let wrong_color = !reversed && cell_bevy.fg != cell_helix.fg;
			let reversed_and_wrong_color = reversed && cell_bevy.fg != cell_helix.bg;

			if wrong_color || reversed_and_wrong_color {
				on_color_changed(
					cell_bevy,
					cell_helix,
					reversed,
					materials,
					commands
				);
			}

			// now spawn new mesh if needed
			let x = (column as f32) * glyph_width;
			let pos = local_position + Vec3::new(x, y, 0.0);

			let wrong_symbol = cell_helix.symbol != cell_bevy.symbol;
			if wrong_symbol {
				println!("cell [{} {}] h [{}] b [{}]", x_cell, y_cell, cell_helix.symbol, cell_bevy.symbol);
				println!("wrong symbol");

				on_symbol_changed(
					pos,
					cell_helix,
					cell_bevy,
					font,
					font_size,
					font_depth,
					text_mesh_cache,
					ttf2_mesh_cache,
					&mut children,
					meshes,
					commands
				);

				println!("after      h [{}] b [{}]", cell_helix.symbol, cell_bevy.symbol);
				println!("----");
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

fn on_symbol_changed(
	pos				: Vec3,
	cell_helix		: &CellHelix,
	cell_bevy		: &mut CellBevy,
	font			: &mut TTFFile,
	font_size		: f32, 
	font_depth		: f32,
	text_mesh_cache	: &mut MeshesMap,
	ttf2_mesh_cache	: &mut TTF2MeshCache,
	children		: &mut Vec<Entity>,
	meshes			: &mut Assets<Mesh>,
	commands		: &mut Commands
) {
    let cache = text_mesh_cache.get(&cell_helix.symbol);
    let cache_found = cache.is_some();
    let space_symbol = cell_helix.symbol == " ";
    if !cache_found && !space_symbol {
		println!("no cache, not space");

		if cell_bevy.entity.is_none() {
			cell_bevy.entity = Some(
				commands.spawn_bundle(
					PbrBundle {
						transform : Transform::from_translation(pos),
						..default()
					}
				)
				.id()
			);
		}
		let mesh_entity_id = cell_bevy.entity.unwrap();

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
		text_mesh_cache.insert(cell_helix.symbol.clone(), mesh_handle);

		cell_bevy.entity = Some(mesh_entity_id);
	} else if !cache_found && space_symbol {
		if let Some(entity) = cell_bevy.entity {
			// remove mesh
			commands.entity(entity).remove::<Handle<Mesh>>();
		}
	} else if let Some(cache) = cache {
		if let Some(entity) = cell_bevy.entity {
			println!("cache, replacing mesh");
			// replace previous mesh with new one
			commands.entity(entity)
				.remove::<Handle<Mesh>>()
				.insert(cache.clone_weak())
				;
		} else {
			// spawn new entity with an existing mesh
			cell_bevy.entity = Some(
				commands.spawn_bundle(PbrBundle {
					mesh : cache.clone_weak(),
					material : cell_bevy.fg_handle.as_ref().unwrap().clone_weak(),
					transform : Transform::from_translation(pos),
					..default()
				})
				.id()
			);
		}
	}
    cell_bevy.symbol = cell_helix.symbol.clone();
}

fn on_color_changed(
	cell_bevy: &mut CellBevy,
	cell_helix: &CellHelix,
	reversed: bool,
	materials: &mut Assets<StandardMaterial>,
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

    cell_bevy.fg_handle = None;
    cell_bevy.bg_handle = None;

    for m in materials.iter() {
		if m.1.base_color == color_fg && cell_bevy.fg_handle.is_none() {
			cell_bevy.fg_handle = Some(materials.get_handle(m.0));
		}
		if m.1.base_color == color_bg && cell_bevy.bg_handle.is_none() {
			cell_bevy.bg_handle = Some(materials.get_handle(m.0));
		}

		if cell_bevy.fg_handle.is_some() && cell_bevy.bg_handle.is_some() {
			break;
		}
	}

    if None == cell_bevy.fg_handle {
		cell_bevy.fg_handle = Some(materials.add(
			StandardMaterial {
				base_color : color_fg,
				unlit : true,
				..default()
			}
		));
	}
    if None == cell_bevy.bg_handle {
		cell_bevy.bg_handle = Some(materials.add(
			StandardMaterial {
				base_color : color_bg,
				unlit : true,
				..default()
			}
		));
	}

    // replace material to reflect changed color
    if let Some(cell_bevy_entity) = cell_bevy.entity {
		commands.entity		(cell_bevy_entity)
		.remove::<Handle<StandardMaterial>>()
		.insert(cell_bevy.fg_handle.as_ref().unwrap().clone_weak())
		;
	}
}

pub fn cursor(
	root_entity		: Entity,
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

	let local_position		= Vec3::ZERO;

	let mut children : Vec<Entity> = Vec::new();

	let width				= surface_helix.area.width;
	let content_helix 		= &surface_helix.content;
	let content_bevy 		= &surface_bevy.content;

	// move background quad
	if let Some(cursor_entity) = cursor.entity && cursor.easing_accum < 1.0 {
		let column_offset 	= (cursor.x as f32) * glyph_width;
		let target_x 		= column_offset + (glyph_width / 2.0) + lbearing;
		let target_y 		= calc_vertical_offset(cursor.y as f32) + (glyph_height / 2.0) + (ybounds[0] * font_size_scalar);

		let target_pos		= local_position + Vec3::new(target_x, target_y, -0.25 / 72.);

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
use bevy				:: prelude :: { * };

use crate				:: bevy_ab_glyph::{ ABFonts };

use super				:: { * };

use helix_tui			:: { buffer :: Cell as CellHelix };

use helix_view::graphics::Color as HelixColor;
use helix_view::graphics::Style as HelixStyle;

pub struct Quad {
	pub x			: f32,
	pub y			: f32,
	pub z			: f32,
	pub glyph_width	: f32,
	pub height		: f32,
	pub v_down_offset : f32,
	pub row			: u32,
	pub column		: u32,
	pub color		: HelixColor,
	pub length		: u32,
}

impl Default for Quad {
	fn default() -> Self {
		Self {
			x			: 0.0,
			y			: 0.0,
			z			: 0.0,
			glyph_width	: 0.0,
			height		: 0.0,
			v_down_offset : 0.0,
			row			: 0,
			column		: 0,
			color		: HelixColor::Cyan,
			length		: 0,
		}
	}
}

impl Quad {
	pub fn size(&self) -> Vec2 {
		Vec2::new(self.width(), self.height)
	}
	
	pub fn width(&self) -> f32 {
		self.glyph_width * self.length as f32
	}
	
	pub fn position(&self) -> Vec3 {
		let x = self.x + (self.width() / 2.0);
		let y = self.y + (self.height / 2.0) - self.v_down_offset;
		let z = self.z;
		Vec3::new(x, y, z)
	}
}

pub type Row = Vec<Quad>;

#[derive(Default)]
pub struct RowState {
	pub quad_started : bool,
	pub synced		: bool,
	pub ended		: bool,
}

#[derive(Component, Clone, Debug)]
pub struct QuadDescription {
	pub length		: u32,
	pub row			: u32,
	pub column		: u32,
}

pub fn append_quad<'a>(
	background_style: &HelixStyle,
	
	surface_coords	: &SurfaceCoords,
	row_bevy		: &mut BackgroundQuadRowBevy,
	row_state		: &mut RowState,
	
	quads_row		: &mut Row,
	cell_helix		: &CellHelix,
	used_fonts		: &'a ABFonts<'a>,
	
	helix_colors_cache: &mut HelixColorsCache,
	
	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	
	commands		: &mut Commands,
) -> Vec<Entity>
{
	let background_color = background_style.bg.unwrap();
	let quad_color_helix = cell_helix.bg;
	let is_space	= quad_color_helix == background_color;
	
	let mut quad_entities : Vec<Entity> = Vec::new();
	
    if row_state.quad_started {
		let quad_index	= quads_row.len() - 1;
		let quad		= quads_row.last_mut().unwrap();
	
		// if word ended check if it's different from what we already have spawned and spawn it or re-use existing entity to attach a different mesh to it
		let different_color = quad_color_helix != quad.color;
		let quad_ended	= is_space || row_state.ended || different_color;
		
		if (quad_ended && row_state.ended) || !quad_ended {
			quad.length += 1;
		}
		
		if quad_ended {
			let entity = on_quad_ended(
				quad_index,
				quad,
				surface_coords,
				row_bevy,
				row_state,
				helix_colors_cache,
				mesh_assets,
				material_assets,
				commands
			);
			
			if let Some(to_add) = entity {
				quad_entities.push(to_add)
			}
		}
	}
	
    if !is_space && !row_state.quad_started {
		row_state.quad_started = true;
		
		let font		= used_fonts.main;
		let v_advance	= font.vertical_advance();
		let h_advance	= font.horizontal_advance(&String::from("a")); // in monospace font every letter should be of the same width so we pick 'a'
		let v_down_offset = font.vertical_down_offset();
	
		let mut quad	= Quad::default();
		quad.glyph_width = h_advance;
		quad.height		= v_advance;
		quad.v_down_offset = v_down_offset;
		
		quad.x			= surface_coords.x;
		quad.y			= surface_coords.y;
		quad.z			= SurfaceBevy::quad_z_offset(font);
		
		quad.row		= surface_coords.row;
		quad.column		= surface_coords.column;
		quad.color		= quad_color_helix;
	
		quad.length		= 1;
	
		if row_state.ended {
			let entity = on_quad_ended(
				quads_row.len(),
				&quad,
				surface_coords,
				row_bevy,
				row_state,
				helix_colors_cache,
				mesh_assets,
				material_assets,
				commands
			);
			
			if let Some(to_add) = entity {
				quad_entities.push(to_add)
			}
		}

		quads_row.push	(quad);
	}
	
	let quads_cnt		= quads_row.len();
    if row_state.ended && (!row_state.synced || quads_row.len() == 0 || quads_cnt < row_bevy.len()) {
		cleanup_desync_quad_row(quads_cnt, row_bevy, commands);
	}
	
	return quad_entities;
}

fn on_quad_ended(
	quad_index		: usize,	
	quad 			: &Quad,
	surface_coords	: &SurfaceCoords,
	row_bevy		: &mut BackgroundQuadRowBevy,
	row_state		: &mut RowState,
	
	helix_colors_cache : &mut HelixColorsCache,
	
	mesh_assets		: &mut Assets<Mesh>,
	material_assets	: &mut Assets<StandardMaterial>,
	
	commands		: &mut Commands,
) -> Option<Entity>
{	
	let mut quad_entity : Option<Entity> = None;
	
	row_state.quad_started = false;
		
	if row_state.synced || quad_index == 0 {
		row_state.synced = check_quad_row_sync(quad_index, quad, row_bevy, commands);
	}

	let quad_description = QuadDescription {
		length	: quad.length,
		row		: surface_coords.row,
		column	: surface_coords.column,
	};

	// now spawn new mesh if needed
	if !row_state.synced {
		quad_entity = update_quad_mesh(
			quad_index,
			quad,
			&quad_description,
			row_bevy,
			helix_colors_cache,
			mesh_assets,
			material_assets,
			commands
		);
	}
	
	quad_entity
}

fn update_quad_mesh(
	quad_index			: usize,
	quad 				: &Quad,
	quad_description	: &QuadDescription,
	row_bevy			: &mut BackgroundQuadRowBevy,
	helix_colors_cache	: &mut HelixColorsCache,
	mesh_assets			: &mut Assets<Mesh>,
	material_assets		: &mut Assets<StandardMaterial>,
	commands			: &mut Commands
) -> Option<Entity>
{
	let color			= color_from_helix(quad.color);
	let material_handle = get_helix_color_material_handle(
		color,
		helix_colors_cache,
		material_assets
	);
	
	// spawn new word if it doesnt exist in the row yet
	if quad_index >= row_bevy.len() {
		let quad_entity = spawn::background_quad(quad.position(), quad.size(), Some(&material_handle), mesh_assets, commands);
		
		commands.entity(quad_entity).insert(quad_description.clone());
		
		row_bevy.push(BackgroundQuadBevy {
			entity		: Some(quad_entity),
			length		: quad.length,
			color		: quad.color,
			column		: quad.column
		});
		
		return Some(quad_entity);
	} else {
		let quad_bevy = &mut row_bevy[quad_index];
		
		quad_bevy.length = quad.length;
		quad_bevy.color	= quad.color;
		quad_bevy.column = quad.column;
		
		let quad_mesh_handle = mesh_assets.add(Mesh::from(shape::Quad::new(quad.size())));
		
		let entity = quad_bevy.entity.unwrap();
		fill_quad_entity(entity, quad, quad_description, &quad_mesh_handle, &material_handle, commands);
		
		return None;
	}
}

fn fill_quad_entity(
	entity			: Entity,
	quad 			: &Quad,
	quad_description: &QuadDescription,
	mesh_handle		: &Handle<Mesh>,
	material_handle	: &Handle<StandardMaterial>,
	commands		: &mut Commands
)
{
	let transform	= Transform {
		translation	: quad.position(),
		..default()
	};
	
	commands.entity(entity)
		.insert(mesh_handle.clone())
		.insert(material_handle.clone_weak())
		.insert(transform)
		.insert(quad_description.clone())
	;
}

fn check_quad_row_sync(
	quad_index			: usize,
	quad 				: &Quad,
	row_bevy			: &mut BackgroundQuadRowBevy,
	commands			: &mut Commands
) -> bool
{
	let row_len			= row_bevy.len();
	if quad_index >= row_len {
		return false;
	}

	// check if it's the same word as we already have in row_bevy and return if so
	let quad_bevy = &row_bevy[quad_index];
	if quad_bevy.length == quad.length && quad_bevy.column == quad.column && quad_bevy.color == quad.color {
		return true;
	}
	
	// as we're desynced invalidate all remaining meshes, transforms and materials. Just keep entities to avoid respawning
	// TODO: we can be smarter here and clean up only current word since next word can be valid just with wrong transform and/or material
	for i in quad_index .. row_len {
		let quad_bevy = &row_bevy[i];
		if let Some(entity) = quad_bevy.entity {
			commands.entity(entity)
				.remove::<Handle<Mesh>>()
				.remove::<Handle<StandardMaterial>>()
				.remove::<Transform>()
				.remove::<QuadDescription>()
			;
		}
	}
	
	return false;
}

fn cleanup_desync_quad_row(
	quad_index			: usize,
	row_bevy			: &mut BackgroundQuadRowBevy,
	commands			: &mut Commands
)
{
	let row_len			= row_bevy.len();
	if quad_index >= row_len {
		return;
	}
	
	for i in quad_index .. row_len {
		let quad_bevy = &row_bevy[i];
		if let Some(entity) = quad_bevy.entity {
			commands.entity(entity).despawn_recursive();
		}
	}
	
	assert!(quad_index <= row_len);
	row_bevy.truncate(quad_index);
}
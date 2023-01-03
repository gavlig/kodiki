use bevy				:: prelude :: { * };

use crate				:: bevy_ab_glyph::{ ABGlyphFont, TextMeshesCache };

use super				:: { * };


use helix_view			:: { Theme };

use helix_view::graphics::Color as HelixColor;

#[derive(Default, Resource)]
pub struct CursorBevy {
	pub entity  	: Option<Entity>,
	pub color   	: Color,
	pub x       	: u32,
	pub y       	: u32,
	pub kind    	: helix_view::graphics::CursorKind,

	pub easing_accum : f32,
}

impl CursorBevy {
	pub fn spawn(
		cursor			: &mut CursorBevy,
		
		surface_entity	: Entity,
		font			: &ABGlyphFont,
	
		text_meshes_cache : &mut TextMeshesCache,
		helix_colors_cache : &mut HelixColorsCache,
	
		material_assets	: &mut Assets<StandardMaterial>,
		mesh_assets		: &mut ResMut<Assets<Mesh>>,
		commands		: &mut Commands
	) {
		let cursor_color_fg	= color_from_helix(HelixColor::Magenta);
		let material_handle	= get_helix_color_material_handle(cursor_color_fg, helix_colors_cache, material_assets);
		
		let v_advance		= font.vertical_advance();
		let h_advance		= font.horizontal_advance_mono();
	
		let glyph_width		= h_advance;
		let glyph_height	= v_advance;
	
		let cursor_z		= SurfaceBevy::cursor_z_offset(font);
	
		let quad_width		= glyph_width;
		let quad_height		= glyph_height;
		let quad_pos		= Vec3::new(0., 0., cursor_z);
	
		// spawn dedicated quad for cursor
		let quad_entity_id	= 
		spawn::glyph_quad(
			quad_pos,
			Vec2::new(quad_width, quad_height),
			text_meshes_cache,
			mesh_assets,
			commands
		);
	
		commands.entity(quad_entity_id).insert(material_handle.clone_weak());
	
		commands.entity(surface_entity).add_child(quad_entity_id);
	
		cursor.entity 		= Some(quad_entity_id);
		cursor.color		= cursor_color_fg;
	}
	
	pub fn update(
		&mut self,
		theme				: &Theme,

		helix_colors_cache	: &mut HelixColorsCache,

		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands
	) {
		let cursor_theme = theme.get("ui.cursor");
		if cursor_theme.bg.is_none() {
			return;
		}

		let cursor_color_fg	= color_from_helix(cursor_theme.bg.unwrap());
		let material_handle	= get_helix_color_material_handle(cursor_color_fg, helix_colors_cache, material_assets);
		
		if self.color != cursor_color_fg {
			commands.entity	(self.entity.unwrap())
				.remove::<Handle<StandardMaterial>>()
				.insert(material_handle.clone_weak())
			;
			self.color	= cursor_color_fg;
		}
	}
	
	pub fn animate(
		&mut self,
		q_cursor_transform	: &mut Query<&mut Transform>,
		
		font				: &ABGlyphFont,
		time				: &Res<Time>,
	
		row_offset			: u32,
		app					: &mut NonSendMut<Application>,
	) {
		let editor_area     = app.area;
		let (cursor_pos, cursor_kind) = app.cursor(editor_area);
		if let Some(cursor_pos) = cursor_pos {
			// cursor position changed so we reset easing timer
			if self.x != cursor_pos.0 as u32
			|| self.y != cursor_pos.1 as u32
			{
				self.easing_accum = 0.0;
			}
	
			self.x			= cursor_pos.0 as u32;
			self.y			= cursor_pos.1 as u32 + row_offset;
			self.kind		= cursor_kind;
		}
	
		let v_advance		= font.vertical_advance();
		let h_advance		= font.horizontal_advance_mono();
		let v_down_offset	= font.vertical_down_offset();
	
		let glyph_width		= h_advance;
		let glyph_height	= v_advance;
	
		let cursor_z		= -font.depth_scaled() + (font.depth_scaled() / 4.0);
	
		// move background quad
		if self.entity.is_some() && self.easing_accum < 1.0 {
			let column_offset = (self.x as f32) * h_advance;
			let row_offset	= ((self.y as f32) * -v_advance) + v_advance; 
	
			let target_x 	= column_offset	+ (glyph_width / 2.0);
			let target_y 	= row_offset	- (glyph_height / 2.0) - v_down_offset;
	
			let target_pos	= Vec3::new(target_x, target_y, cursor_z);
	
			let delta_seconds = time.delta_seconds();
			let delta_accum	= delta_seconds / /*cursor_easing_seconds*/ 0.0001;
	
			let cursor_entity = self.entity.unwrap();
			let mut cursor_transform = q_cursor_transform.get_mut(cursor_entity).unwrap();
	
			self.easing_accum = (self.easing_accum + delta_accum).min(1.0);
			cursor_transform.translation = cursor_transform.translation.lerp(target_pos, self.easing_accum);
		}
	}
}
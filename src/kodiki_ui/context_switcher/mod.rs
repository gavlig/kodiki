use bevy :: prelude :: *;

use bevy_rapier3d :: prelude :: *;
use bevy_tweening :: *;

use crate :: bevy_ab_glyph :: ABGlyphFonts;
use crate :: z_order;
use crate :: kodiki_ui :: { * , color :: * , tween_lens :: * , raypick :: * };

use std :: collections :: VecDeque;
use std :: time :: Duration;

pub mod systems;

#[derive(Component)]
pub struct ContextSwitcherHighlight;

#[derive(Component)]
pub struct ContextSwitcher {
	pub width				: f32,
	pub entry_height		: f32,
	pub margin				: f32,
}

#[derive(Component)]
pub struct ContextSwitcherEntry {
	pub is_active			: bool,
	pub is_triggered		: bool, // processing logic should set it to false
	pub glyph				: String,
	pub hint				: String,
	pub hint_hotkey			: String,
	pub quad_color			: Color,
	pub switcher_entity		: Option<Entity>,
	pub callback			: Option<Box<dyn FnMut() + Send + Sync>>,
}

impl ContextSwitcherEntry {
	pub fn highlight(
		&self,
		entity		: Entity,
		transform	: &Transform,
		color_materials_cache	: &mut ColorMaterialsCache,
		material_assets			: &mut Assets<StandardMaterial>,
		commands	: &mut Commands,
	) {
		let duration 		= Duration::from_millis(250);
		let ease			= EaseFunction::SineIn;

		let scale			= 1.07;
		let hovered_scale	= transform.scale * scale;

		let tween = Tween::new(
			ease,
			duration,
			TransformLens {
				start : transform.clone(),
				end : Transform {
					translation : transform.translation,
					scale : hovered_scale,
					..default()
				}
			}
		);

		let new_quad_color = get_color_wmodified_lightness(self.quad_color, 0.1);

		let quad_material_handle = get_color_material_handle(
			new_quad_color,
			color_materials_cache,
			material_assets
		);

		commands.entity(entity).insert(ContextSwitcherHighlight);

		commands.entity(entity)
			.insert(Animator::new(tween))
			.insert(quad_material_handle.clone_weak())
		;
	}

	pub fn unhighlight(
		&self,
		entity		: Entity,
		transform	: &Transform,
		color_materials_cache	: &mut ColorMaterialsCache,
		material_assets			: &mut Assets<StandardMaterial>,
		commands	: &mut Commands,
	) {
		let duration 	= Duration::from_millis(500);
		let ease		= EaseFunction::ExponentialOut;

		let tween = Tween::new(
			ease,
			duration,
			TransformLens {
				start : transform.clone(),
				end : Transform {
					translation : transform.translation,
					scale : Vec3::ONE,
					..default()
				}
			}
		);

		let quad_material_handle = get_color_material_handle(
			self.quad_color,
			color_materials_cache,
			material_assets
		);

		commands.entity(entity)
			.remove::<ContextSwitcherHighlight>()
			.insert(Animator::new(tween))
			.insert(quad_material_handle.clone_weak())
		;
	}

	pub fn glyph_spawn_request(&self) -> CommonString3dSpawnParams {
		self.glyph_spawn_request_color(Color::ANTIQUE_WHITE)
	}

	pub fn glyph_spawn_request_color(&self, color: Color) -> CommonString3dSpawnParams {
		CommonString3dSpawnParams {
			string		: self.glyph.clone(),
			color,
			transform	: Transform {
				translation : Vec3::new(0.0, 0.0, z_order::surface::text()),
				scale : Vec3::ONE * 1.5,
				..default()
			},
			row			: 0.5,
			col			: -0.6,
			..default()
		}
	}

	pub fn glyph_spawn_callback() -> Box<dyn Fn(Entity, Entity, &mut Commands) + Send + Sync> {
		Box::new(
			|owner_entity: Entity, glyph_entity, commands: &mut Commands| {
				commands.entity(owner_entity).insert(ContextSwitcherGlyph { entity: glyph_entity });
			}
		)
	}
}

#[derive(Component)]
pub struct ContextSwitcherGlyph {
	pub entity : Entity,
}

const CONTEXT_SWITCH_WIDTH	: f32 = 0.2;
const CONTEXT_SWITCH_HEIGHT	: f32 = 0.2;
const CONTEXT_SWITCH_MARGIN	: f32 = 0.03;

impl Default for ContextSwitcher {
	fn default() -> Self {
		Self {
			width			: CONTEXT_SWITCH_WIDTH,
			entry_height	: CONTEXT_SWITCH_HEIGHT,
			margin			: CONTEXT_SWITCH_MARGIN,
		}
	}
}

impl ContextSwitcher {
	pub fn new_entry(glyph: String, hint: String, hint_hotkey: String) -> ContextSwitcherEntry {
		ContextSwitcherEntry {
			is_active: false,
			is_triggered: false,
			glyph,
			hint,
			hint_hotkey,
			quad_color: Color::CYAN,
			switcher_entity: None,
			callback: None,
			// callback: if let Some(cb) = callback { Some(Box::new(cb)) } else { None },
		}
	}

	pub fn spawn(
		width			: f32,
		mut entries		: VecDeque<ContextSwitcherEntry>,
		mesh_assets		: &mut Assets<Mesh>,
		material_assets	: &mut Assets<StandardMaterial>,
		fonts			: &ABGlyphFonts,
		commands		: &mut Commands,
	) -> Vec<Entity>{
		debug_assert!(entries.len() > 0);

		let switcher	= ContextSwitcher { width, ..default() };

		let entry_width	= switcher.width;
		let entry_height = switcher.entry_height;

		let switcher_entity = commands.spawn((
			switcher,
			TransformBundle::default(),
			VisibilityBundle::default(),
		)).id();

		let column_width = fonts.main.horizontal_advance_mono();
		let row_height = fonts.main.vertical_advance();

		let entry_x_offset = width / 2.0 + column_width;
		let entry_y_offset = -row_height / 2.0;

		let mut output_vec = Vec::new();

		while let Some(mut entry) = entries.pop_front() {
			let quad_size = Vec2::new(entry_width, entry_height);
			let quad_mesh_handle = mesh_assets.add(Rectangle::from_size(quad_size));
			let quad_material_handle = material_assets.add(StandardMaterial {
				base_color : entry.quad_color,
				unlit : true,
				..default()
			});

			entry.switcher_entity = Some(switcher_entity);

			let entry_entity = commands.spawn((
				PbrBundle {
					mesh		: quad_mesh_handle,
					material	: quad_material_handle,
					transform	: Transform::from_translation(Vec3::Z * z_order::context_switcher()),
					..default()
				},
				String3dSpawnRequest {
					common : entry.glyph_spawn_request(),
					callback : Some(ContextSwitcherEntry::glyph_spawn_callback()),
					..default()
				},
				HintHotkey {
					common : CommonString3dSpawnParams {
						string		: entry.hint_hotkey.clone(),
						transform	: Transform::from_translation(Vec3::new(width / 2.0, 0.0, z_order::surface::text())),
						row			: 0.5,
						col			: 1.0,
						..default()
					},
					..default()
				},
				HintHover {
					common : CommonString3dSpawnParams {
						string: entry.hint.clone(),
						transform: Transform::from_translation(Vec3::new(entry_x_offset, entry_y_offset, z_order::surface::text())),
						..default()
					},
					..default()
				},
				RigidBody		:: Fixed,
				Collider		:: cuboid(quad_size.x / 2., quad_size.y / 2., z_order::thickness() / 2.),
				RaypickHover	:: default()
			)).id();

			commands.entity(entry_entity).insert(entry);

			output_vec.push(entry_entity);

			commands.entity(switcher_entity).add_child(entry_entity);
		}

		output_vec
	}
}

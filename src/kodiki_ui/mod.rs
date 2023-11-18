use bevy :: prelude :: *;
use bevy :: utils 	:: HashMap;
use bevy_reader_camera :: *;

pub mod utils;
pub mod tween_lens;
pub mod color;
pub mod spawn;
pub mod text_background_quad;
pub mod context_switcher;
pub mod text_cursor;
pub mod text_surface;
pub mod resizer;
pub mod raypick;
pub mod popups;

mod systems;

use text_cursor :: *;
use raypick		:: *;
use popups		:: *;

pub type MaterialsMap = HashMap<String, Handle<StandardMaterial>>;

#[derive(Default, Resource)]
pub struct ColorMaterialsCache {
	pub materials: MaterialsMap,
}

#[derive(Resource, Default)]
pub struct DraggingState {
	pub entity : Option<Entity>,
}

impl DraggingState {
	pub fn is_active(&self) -> bool {
		self.entity.is_some()
	}

	pub fn set_active(&mut self, entity: Entity) {
		self.entity = Some(entity);
	}

	pub fn unset_active(&mut self) {
		self.entity = None;
	}
}

#[derive(Component, Clone, Debug)]
pub struct WordSubEntities {
	pub mesh_entity		: Entity,
	pub collision_entity: Entity,
}

#[derive(Component)]
pub struct StringMeshAttached {
	pub id		: usize,
	pub string	: String,
	pub entity	: Entity,
	pub despawn_requested : bool,
}

impl Default for StringMeshAttached {
	fn default() -> Self {
        Self {
			id: 0,
			string: "[STRING_NOT_SET]".into(),
			entity: Entity::from_raw(0),
			despawn_requested: false
		}
    }
}

#[derive(Default, Clone)]
pub struct CommonString3dSpawnParams {
	pub string		: String,
	pub color		: Color,
	pub background_color : Option<Color>,
	pub transform	: Transform,
	pub row			: f32,
	pub col			: f32,
	pub id			: usize,
}

#[derive(Component, Default)]
pub struct String3dSpawnRequest {
	pub common		: CommonString3dSpawnParams,
	pub add_attached_component : bool,
	pub callback	: Option<Box<dyn Fn(Entity, Entity, &mut Commands) + Send + Sync>>,
}

impl String3dSpawnRequest {
	pub fn add_self_to(
		self,
		owner_entity : Entity,
		q_string_mesh_attached : &Query<&StringMeshAttached>,
		commands : &mut Commands
	) -> bool {
		let no_previous_attaches = !q_string_mesh_attached.get(owner_entity).is_ok();
		if no_previous_attaches {
			commands.entity(owner_entity).insert(self);
		}

		return no_previous_attaches
	}

	pub fn add_self_to_qmut(
		self,
		owner_entity : Entity,
		q_string_mesh_attached : &Query<&mut StringMeshAttached>,
		commands : &mut Commands
	) -> bool {
		let no_previous_attaches = !q_string_mesh_attached.get(owner_entity).is_ok();
		if no_previous_attaches {
			commands.entity(owner_entity).insert(self);
		}

		return no_previous_attaches
	}}

#[derive(Component, Default)]
pub struct HintHotkey {
	pub common		: CommonString3dSpawnParams,
	pub active		: bool,
}

impl HintHotkey {
    pub const ID: usize = 1;
}

#[derive(Component, Default)]
pub struct HintHover {
	pub common		: CommonString3dSpawnParams,
	pub active		: bool,
}

impl HintHover {
    pub const ID: usize = 2;
}

#[derive(Resource, Default)]
pub struct KodikiUI {
	pub dark_theme : bool,
	pub context_switch_color : Color,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
pub struct KodikiUISystems;

pub struct KodikiUIPlugin;

impl Plugin for KodikiUIPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(KodikiUI				:: default())
			.insert_resource(ColorMaterialsCache	:: default())
			.insert_resource(CursorVisualAsset		:: default())
			.insert_resource(DraggingState			:: default())
			.insert_resource(Raypick				:: default())
			.insert_resource(Popups					:: default())

			// raypick relies on camera transform so it has to be calculated only after camera gets updated
			.add_system(
				raypick::systems::cast_raypick
					.in_base_set(CoreSet::PreUpdate)
					.after(ReaderCameraUpdate)
			)
			.add_systems(
				(
					systems::process_string_spawn_requests,
					systems::cleanup_string_mesh_attached,
					systems::update_hotkey_hints,
					systems::process_hover_hints,
					systems::cleanup_hover_hints,
				).in_set(KodikiUISystems)
			)
			.add_systems(
				(
					text_background_quad::systems::update_color,
					text_background_quad::systems::update_transform,
					text_cursor::systems::update,
					text_surface::systems::spawn_words,
					text_surface::systems::spawn_coloring_lines,
					resizer::systems::input_mouse,
					resizer::systems::update_color,
					resizer::systems::highlight_hovered,
					context_switcher::systems::update_position,
					context_switcher::systems::update_color,
					context_switcher::systems::mouse_input,
					context_switcher::systems::highlights_cleanup,
				).in_set(KodikiUISystems)
			)
 		;
	}
}

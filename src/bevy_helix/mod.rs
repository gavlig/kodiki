use bevy :: prelude :: *;
use bevy :: utils :: HashMap;
use bevy_tweening :: *;

use helix_view :: { DocumentId, Document };

use crate :: {
	kodiki :: { AppMode, AppContext, run_condition },
	kodiki_ui :: KodikiUISystems
};

use std :: time :: { Instant, Duration };

pub mod spawn;
pub mod tween_lens;

pub mod helix_app;
pub use helix_app :: *;

pub mod surface;
use surface :: *;

mod input;
pub mod utils;

mod search;
use search :: *;

mod minimap;
use minimap :: *;

mod systems_util;
mod systems;

#[derive(Resource)]
pub struct BevyHelixSettings {
	pub key_press_init_delay_seconds	: f32,
	pub key_press_long_delay_seconds	: f32,
	pub double_click_delay_seconds		: f32,
}

impl Default for BevyHelixSettings {
	fn default() -> Self {
		Self {
			key_press_init_delay_seconds	: 0.2,
			key_press_long_delay_seconds	: 0.04,
			double_click_delay_seconds		: 0.1,
		}
	}
}


pub struct TweenPoint {
	pub pos			: Vec3,
	pub delay		: Duration,
	pub ease_function : EaseFunction,
}

#[derive(Component)]
pub struct GotoDefinitionHighlight;

#[derive(Resource, Deref, DerefMut)]
pub struct TokioRuntime(pub tokio::runtime::Runtime);

#[derive(Clone, Copy)]
pub struct KeyPressTiming {
	pub init				: Instant,
	pub long				: Option<Instant>,
}

#[derive(Resource, Default)]
pub struct ArrowKeysState {
	pub last_event_up		: Option<KeyPressTiming>,
	pub last_event_down		: Option<KeyPressTiming>,
	pub last_event_left		: Option<KeyPressTiming>,
	pub last_event_right	: Option<KeyPressTiming>,
}

#[derive(Resource, Default)]
pub struct MousePosState {
	pub row : u16,
	pub col : u16,
	pub surface_name : String,
}

#[derive(Resource, Default)]
pub struct MouseButtonState {
	pub last_clicked		: HashMap<MouseButton, Instant>
}

impl MouseButtonState {
	pub fn is_double_click(&self, btn: &MouseButton, delay: f32) -> bool {
		if let Some(clicked_instant) = self.last_clicked.get(btn) {
			let now = Instant::now();
			now.duration_since(*clicked_instant).as_secs_f32() <= delay
		} else {
			false
		}
	}
}

#[derive(Resource)]
pub struct MouseHoverState {
	timer			: Timer,
	row				: u16,
	col				: u16,
	syntax_node_id	: Option<usize>,
}

impl MouseHoverState {
	pub fn new() -> Self {
		Self {
			timer	: Timer::from_seconds(0.5, TimerMode::Once),
			row		: 0,
			col		: 0,
			syntax_node_id: None,
		}
	}
}

impl Default for MouseHoverState {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone, Copy, Debug)]
pub enum HighlightKind {
	Diagnostic,
	Search,
	Selection,
	SelectionSearch,
	Cursor
}

impl From<SearchKind> for HighlightKind {
	fn from(kind: SearchKind) -> HighlightKind {
		match kind {
			SearchKind::Common => HighlightKind::Search,
			SearchKind::Selection => HighlightKind::SelectionSearch
		}
	}
}

pub struct Highlights<CacheType> {
	pub cache				: Option<CacheType>,
	pub entities			: Vec<Entity>,
}

impl<CacheType> Default for Highlights<CacheType> {
	fn default() -> Self {
		Self {
			cache			: None,
			entities		: Vec::new(),
		}
	}
}

pub type VersionType = usize;

#[derive(Default, Clone, Debug)]
pub struct SyncDataDoc {
	pub id		: DocumentId,
	pub theme	: String,
	pub version	: VersionType,
	pub horizontal_offset : Option<usize>,
}

impl SyncDataDoc {
	pub fn outdated(&self, doc: &Document, theme: &str) -> bool {
		self.id != doc.id() || self.version != doc.version() || self.theme.as_str() != theme
	}
}

#[derive(Default, Clone, Debug)]
pub struct SyncDataString {
	pub doc		: SyncDataDoc,
	pub string	: String,
}

#[derive(Default, Clone, Debug)]
pub struct SyncDataDiagnostics {
	pub doc		: SyncDataDoc,
	pub diagnostics_version : VersionType,
}

// System Sets

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct TweenEvents;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct HelixRender;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct HelixEvents;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ManageSurfaces;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct UpdateCursor;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct UpdateMain;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct UpdateSecondary;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ContextSwitch;

pub struct BevyHelixPlugin;

impl Plugin for BevyHelixPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(MatchesMapCache		:: default())
			.insert_resource(BevyHelixSettings		:: default())
			.insert_resource(ArrowKeysState			:: default())
			.insert_resource(MousePosState			:: default())
			.insert_resource(MouseButtonState		:: default())
			.insert_resource(MouseHoverState		:: default())
			.insert_resource(SurfacesMapHelix		:: default())
			.insert_resource(WordsToSpawn			:: default())
			.insert_resource(ColoringLinesToSpawn	:: default())

			.insert_resource(TokioRuntime {
				0: tokio::runtime::Builder::new_multi_thread()
				.enable_all()
				.build()
				.unwrap()
			})

			//
			// Long chain of consecutive system sets representing the stages of frame
			//

			.configure_set(
				TweenEvents.in_base_set(CoreSet::Update)
				.run_if(run_condition::text_editor_context)
			)
			.configure_set(
				HelixEvents.in_base_set(CoreSet::Update)
				.after(TweenEvents)
				.run_if(run_condition::text_editor_context_no_fly)
			)
			.configure_set(
				HelixRender.in_base_set(CoreSet::Update)
				.after(HelixEvents)
				.run_if(run_condition::text_editor_context)
			)
			.configure_set(
				ManageSurfaces.in_base_set(CoreSet::Update)
				.after(HelixRender)
				.run_if(run_condition::text_editor_context_no_fly)
			)
			.configure_set(
				UpdateCursor.in_base_set(CoreSet::Update)
				.after(ManageSurfaces)
				.run_if(run_condition::text_editor_context_no_fly)
			)
			.configure_set(
				UpdateMain.in_base_set(CoreSet::Update)
				.after(UpdateCursor)
				.run_if(run_condition::text_editor_context_no_fly)
			)
			.configure_set(
				UpdateSecondary.in_base_set(CoreSet::Update)
				.after(UpdateMain)
				.run_if(run_condition::text_editor_context_no_fly)
			)
			.configure_set(
				ContextSwitch.in_base_set(CoreSet::Update)
				.after(UpdateSecondary)
				.before(KodikiUISystems)
				.run_if(run_condition::main_app_mode)
			)

			//
			// Populating sets with systems
			//

			.add_systems(
				(
					systems::startup_app,
					systems::startup_spawn
				)
				.chain()
				.in_schedule(OnExit(AppMode::AssetsLoaded))
			)
			.add_systems(
				(
					systems::helix_mode_tween_events,
					minimap::systems::update_click_point
				).in_set(TweenEvents)
			)
			.add_systems(
				(
					systems::input_mouse,
					systems::input_keyboard,
					systems::mouse_last_clicked,
					systems::mouse_hover,
					systems::mouse_goto_definition,
					systems::update_editor_resizer,
					minimap::systems::input_mouse,
					minimap::systems::input_mouse_bookmark,
				).in_set(HelixEvents)
			)
			.add_systems(
				(
					systems::camera_update,
					systems::render_helix
				)
				.chain()
				.in_set(HelixRender)
			)
			.add_systems(
				(
					systems::manage_surfaces,
					systems::manage_cursors
				).in_set(ManageSurfaces)
			)
			.add_system(
				systems::update_cursor
				.in_set(UpdateCursor)
			)
			.add_systems(
				(
					systems::update_background_color,
					systems::update_search_matches,
					systems::update_selection_search_matches,
					surface::systems::update,
					minimap::systems::update,
					minimap::systems::handle_render_tasks,
				).in_set(UpdateMain)
			)
			.add_systems(
				(
					systems::helix_mode_effect,
					systems::update_debug_stats,

					surface::systems::spawn_words,
					surface::systems::spawn_coloring_lines,
					surface::systems::highlight_insert_mode,
					surface::systems::update_diagnostics_highlights,
					surface::systems::update_search_highlights,
					surface::systems::update_selection_search_highlights,
					surface::systems::update_selection_highlights,
					surface::systems::update_cursor_highlights,
					surface::systems::update_background_color,
					surface::systems::update_size,
					surface::systems::update_transform,
				).in_set(UpdateSecondary)
			)
			.add_systems(
				(
					minimap::systems::update_bookmarks,
					minimap::systems::update_diagnostics_highlights,
					minimap::systems::update_search_highlights,
					minimap::systems::update_selection_search_highlights,
					minimap::systems::update_selection_highlights,
					minimap::systems::update_transform,
					minimap::systems::reveal_hovered_bookmark,
					minimap::systems::update_minimap_scroll_animation,
				).in_set(UpdateSecondary)
			)
			.add_systems(
				(
					component_animator_system::<MinimapScrollAnimation>,
				).in_set(AnimationSystem::AnimationUpdate)
			)
			.add_systems(
				(
					systems::on_context_switch_out,
				)
				.in_set(ContextSwitch)
				.in_schedule(OnExit(AppContext::CodeEditor))
			)
			.add_systems(
				(
					systems::on_context_switch_in,
				)
				.in_set(ContextSwitch)
				.in_schedule(OnEnter(AppContext::CodeEditor))
			)

			// independent systems
			.add_systems(
				(
					systems::animations_keepalive,
					systems::animations_cleanup_components,
				)
				.in_base_set(CoreSet::PostUpdate)
			)
			// tokio events processing is kept alive even when helix is not in focus to keep Helix updated
			.add_system(
				systems::tokio_events.in_set(OnUpdate(AppMode::Main))
			)

			.add_system(systems::on_window_close_requested)
			.add_system(systems::exit_app)
 		;
	}
}
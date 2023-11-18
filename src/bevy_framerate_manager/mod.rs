use bevy :: prelude :: *;
use bevy_tweening	:: *;
use bevy_rapier3d	:: prelude :: *;

use std :: time :: Duration;

use crate :: kodiki_ui :: { *, color :: *, tween_lens :: *, raypick :: * };

pub mod systems;
pub mod conditions;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FramerateMode {
	Idle,
	Active,
	Smooth
}

#[derive(Resource)]
pub struct FramerateManager {
	idle_timer					: Timer,
	mode						: FramerateMode,
	active_mode_requested		: (bool, String),
	smooth_mode_requested		: (bool, String),

	idle_frame_duration			: Duration,
	working_frame_duration		: Duration,
	active_frame_duration		: Duration,
	smooth_frame_duration		: Duration,

	camera_move_requested		: bool,
	camera_zooming				: bool,
	camera_moving				: bool,

	any_key_pressed				: bool,
	any_mouse_button_pressed 	: bool,
	any_scroll_event			: bool,

	// for debug
	idle_framerate_color		: Color,
	active_framerate_color		: Color,
	smooth_framerate_color		: Color,
	background_color			: Color,

	logs						: Vec<String>,
	clear_logs_on_next_entry	: bool,
}

impl Default for FramerateManager {
	fn default() -> Self {
		Self {
			idle_timer					: Timer::from_seconds(0.25, TimerMode::Once),
			mode						: FramerateMode::Smooth,
			active_mode_requested		: (false, String::from("[EMPTY]")),
			smooth_mode_requested		: (false, String::from("[EMPTY]")),
			idle_frame_duration			: Duration::from_millis(1000 / 5), // 5 fps
			working_frame_duration		: Duration::from_millis(1000 / 30), // at least 30 fps is expected for app to work
			active_frame_duration		: Duration::from_millis(1000 / 60), // 60 fps
			smooth_frame_duration		: Duration::ZERO, // MAX fps

			camera_move_requested		: false,
			camera_zooming				: false,
			camera_moving				: false,

			any_key_pressed				: false,
			any_mouse_button_pressed 	: false,
			any_scroll_event			: false,

			idle_framerate_color		: Color::hex("1ff688").unwrap(),
			active_framerate_color		: Color::hex("1feef6").unwrap(),
			smooth_framerate_color		: Color::hex("b15efb").unwrap(), // 1f9cf6
			background_color			: Color::hex("141a20").unwrap(),

			logs						: Vec::new(),
			clear_logs_on_next_entry	: false,
		}
	}
}

impl FramerateManager {
	pub fn clear_internal_state(&mut self) {
		self.camera_move_requested		= false;
		self.camera_zooming				= false;
		self.camera_moving				= false;

		self.any_key_pressed			= false;
		self.any_mouse_button_pressed 	= false;
		self.any_scroll_event			= false;
	}

	pub fn request_active_framerate(&mut self, reason: String) {
		self.active_mode_requested = (true, reason);
	}

	pub fn active_framerate_reason(&self) -> &String {
		&self.active_mode_requested.1
	}

	pub fn active_framerate_requested(&self) -> bool {
		self.active_mode_requested.0
	}


	pub fn request_smooth_framerate(&mut self, reason: String) {
		self.smooth_mode_requested = (true, reason);
	}

	pub fn smooth_framerate_requested(&self) -> bool {
		self.smooth_mode_requested.0
	}

	pub fn smooth_framerate_reason(&self) -> &String {
		&self.smooth_mode_requested.1
	}

	pub fn read_and_apply_active_framerate_request(&mut self) -> bool {
		let buf = self.active_mode_requested.0;
		self.active_mode_requested.0 = false;

		buf
	}

	pub fn read_and_apply_smooth_framerate_request(&mut self) -> bool {
		let buf = self.smooth_mode_requested.0;
		self.smooth_mode_requested.0 = false;
		buf
	}

	pub fn idle_frame_duration(&self) -> Duration {
		self.idle_frame_duration
	}

	pub fn working_frame_duration(&self) -> Duration {
		self.working_frame_duration
	}

	pub fn active_frame_duration(&self) -> Duration {
		self.active_frame_duration
	}

	pub fn smooth_frame_duration(&self) -> Duration {
		self.smooth_frame_duration
	}

	pub fn mode(&self) -> FramerateMode {
		self.mode
	}

	pub fn set_mode(&mut self, framerate_in: FramerateMode) {
		self.mode = framerate_in;
	}

	pub fn current_frame_duration(&self) -> Duration {
		match self.mode {
			FramerateMode::Idle => {
				self.idle_frame_duration
			},
			FramerateMode::Active => {
				self.active_frame_duration
			},
			FramerateMode::Smooth => {
				self.smooth_frame_duration
			}
		}
	}

	pub fn set_mode_and_get_duration(&mut self, framerate_mode_in: FramerateMode) -> Duration {
		self.set_mode(framerate_mode_in);
		self.current_frame_duration()
	}

	pub fn matches_expected_frame_duration(&mut self, frame_duration_in: Duration) -> bool {
		self.current_frame_duration() == frame_duration_in
	}

	pub fn tick_idle_timer(&mut self, time: &Time) {
		self.idle_timer.tick(time.delta());

		if !self.idle_timer.finished() {
			self.log(format!("tick idle timer {:.2}%", self.idle_timer.percent() * 100.));
		}
	}

	pub fn reset_idle_timer(&mut self) {
		self.idle_timer.reset();
	}

	pub fn idle_timer_finished(&self) -> bool {
		self.idle_timer.finished()
	}

	pub fn animations_allowed(&self, time: &Time) -> bool {
		self.mode != FramerateMode::Idle && Duration::from_secs_f32(time.delta_seconds()) <= self.working_frame_duration()
	}

	pub fn idle_framerate_color(&self) -> Color {
		self.idle_framerate_color
	}

	pub fn active_framerate_color(&self) -> Color {
		self.active_framerate_color
	}

	pub fn smooth_framerate_color(&self) -> Color {
		self.smooth_framerate_color
	}

	pub fn current_framerate_color(&self) -> Color {
		match self.mode {
			FramerateMode::Idle => {
				self.idle_framerate_color()
			},
			FramerateMode::Active => {
				self.active_framerate_color()
			},
			FramerateMode::Smooth => {
				self.smooth_framerate_color()
			}
		}
	}

	pub fn background_color(&self) -> Color {
		self.background_color
	}

	pub fn set_camera_state(&mut self, move_requested: bool, is_zooming: bool, is_moving: bool) {
		self.camera_move_requested = move_requested;
		self.camera_zooming = is_zooming;
		self.camera_moving = is_moving;
	}

	pub fn set_input_state(&mut self, any_key: bool, any_mouse: bool, any_scroll: bool) {
		self.any_key_pressed = any_key;
		self.any_mouse_button_pressed = any_mouse;
		self.any_scroll_event = any_scroll;
	}

	pub fn camera_potentially_active(&self) -> bool {
		self.camera_move_requested || self.camera_moving || self.camera_zooming || self.any_scroll_event
	}

	pub fn input_active(&self) -> bool {
		self.any_key_pressed || self.any_mouse_button_pressed || self.any_scroll_event
	}

	pub fn smooth_framerate_condition(&self) -> bool {
		self.camera_potentially_active()
	}

	pub fn active_framerate_condition(&self) -> bool {
		self.any_key_pressed || self.any_mouse_button_pressed || self.any_scroll_event
	}

	pub fn read_and_apply_smooth_framerate_condition(&mut self) -> bool {
		self.read_and_apply_smooth_framerate_request() || self.smooth_framerate_condition()
	}

	pub fn read_and_apply_active_framerate_condition(&mut self) -> bool {
		self.read_and_apply_active_framerate_request() || self.active_framerate_condition()
	}

	pub fn camera_move_requested(&self) -> bool {
		self.camera_move_requested
	}

	pub fn camera_zooming(&self) -> bool {
		self.camera_zooming
	}

	pub fn camera_moving(&self) -> bool {
		self.camera_moving
	}

	pub fn any_key_pressed(&self) -> bool {
		self.any_key_pressed
	}

	pub fn any_mouse_button_pressed(&self) -> bool {
		self.any_mouse_button_pressed
	}

	pub fn any_scroll_event(&self) -> bool {
		self.any_scroll_event
	}

	pub fn clear_logs(&mut self) {
		self.logs.clear();
	}

	pub fn clear_on_next_entry(&mut self) {
		self.clear_logs_on_next_entry = true;
	}

	pub fn log(&mut self, log: String) {
		if self.clear_logs_on_next_entry == true {
			self.clear_logs();
			self.clear_logs_on_next_entry = false;
		}

		self.logs.push(log);
	}

	pub fn logs(&self) -> &Vec<String> {
		&self.logs
	}

}

#[derive(Resource, Default)]
pub struct FramerateDebug {
	pub dot_color		: Color,
	pub dot_translation	: Vec3,
	pub dot_entity		: Option<Entity>,
	pub lines			: Vec<DebugLine>,
	pub lines_entity	: Option<Entity>,
	pub extra_info_enabled: bool,
}

#[derive(PartialEq)]
pub struct DebugLine {
	pub string		: String,
	pub foreground	: Color,
	pub background	: Color,
}

impl Default for DebugLine {
	fn default() -> Self {
        Self {
			string		: "[STRING NOT INITIALIZED]".into(),
			foreground	: Color::DARK_GRAY,
			background	: FramerateManager::default().background_color()
		}
    }
}

impl DebugLine {
	pub fn new(string: String, foreground: Color, background: Color) -> Self {
		Self {
			string,
			foreground,
			background
		}
	}
	
	pub fn new_default_bg(string: String, foreground: Color) -> Self {
		Self {
			string,
			foreground,
			background : Self::default().background
		}
	}
}

impl FramerateDebug {
	pub fn collect_extra_logs(
		manager		: &FramerateManager,
		lines		: &mut Vec<DebugLine>
	) {
		profile_function!();

		let color_logs = if manager.clear_logs_on_next_entry { Color::GRAY } else { Color::ANTIQUE_WHITE };
		lines.push(DebugLine::new_default_bg("=== logs ===".into(), color_logs));

		for log in manager.logs().iter() {
			lines.push(DebugLine::new_default_bg(log.clone(), color_logs));
		}
	}

	pub fn collect_state_logs(
		manager		: &FramerateManager,
		time		: &Time,
		lines		: &mut Vec<DebugLine>
	) {
		profile_function!();

		// current framerate

		let current_framerate_mode = manager.mode();
		let current_frame_duration = manager.current_frame_duration();
		let framerate_string = format!("FPS mode: {:?} [{:.2}ms/{:?}]", current_framerate_mode, time.delta().as_secs_f32() * 1000.0, current_frame_duration);
		let framerate_color = manager.current_framerate_color();
		let background_color = manager.background_color();

		lines.push(DebugLine { string: framerate_string, foreground: framerate_color, background: background_color });

		// camera, input and other states that trigger/affect high fps

		let color_active = manager.active_framerate_color();
		let color_smooth = manager.smooth_framerate_color();
		let color_false = Color::GRAY;

		let val = manager.camera_move_requested();
		lines.push(DebugLine::new_default_bg(
			format!("camera_move_requested:    {val}"),
			if val { color_smooth } else { color_false })
		);

		let val = manager.camera_moving();
		lines.push(DebugLine::new_default_bg(
			format!("camera_moving:            {val}"),
			if val { color_smooth } else { color_false })
		);

		let val = manager.camera_zooming();
		lines.push(DebugLine::new_default_bg(
			format!("camera_zooming:           {val}"),
			if val { color_smooth } else { color_false })
		);

		let val = manager.any_key_pressed();
		lines.push(DebugLine::new_default_bg(
			format!("any_key_pressed:          {val}"),
			if val { color_active } else { color_false })
		);

		let val = manager.any_mouse_button_pressed();
		lines.push(DebugLine::new_default_bg(
			format!("any_mouse_button_pressed: {val}"),
			if val { color_active } else { color_false })
		);

		let val = manager.any_scroll_event();
		lines.push(DebugLine::new_default_bg(
			format!("any_scroll_event:         {val}"),
			if val { color_active } else { color_false })
		);

		let val = !manager.idle_timer_finished();
		lines.push(DebugLine::new_default_bg(
			format!("idle timer active:        {val}"),
			if val { color_active } else { color_false })
		);

		// request reason

		let last_smooth_request_reason = format!("Smooth req reason: {}", manager.smooth_framerate_reason());
		let last_active_request_reason = format!("Active req reason: {}", manager.active_framerate_reason());

		lines.push(DebugLine::new_default_bg(
			last_smooth_request_reason.clone(),
			if manager.smooth_framerate_requested() { manager.smooth_framerate_color() } else { Color::GRAY })
		);
		lines.push(DebugLine::new_default_bg(
			last_active_request_reason.clone(),
			if manager.active_framerate_requested() { manager.active_framerate_color() } else { Color::GRAY })
		);
	}

	pub fn spawn_visualization_logs(
		translation_in	: Vec3,
		row_height		: f32,
		column_width	: f32,
		lines			: &Vec<DebugLine>,
		commands		: &mut Commands,
	) -> Entity {
		profile_function!();

		let mut translation = translation_in;
		translation.x += column_width;

		let lines_entity = commands.spawn((
			TransformBundle {
				local : Transform::from_translation(translation),
				..default()
			},
			VisibilityBundle::default(),
		))
		.id();

		for (line_index, line) in lines.iter().enumerate() {
			let y = (line_index + 1) as f32 * row_height * -1.0;

			let word_entity = commands.spawn((
				TransformBundle {
					local : Transform::from_translation(Vec3::Y * y),
					..default()
				},
				VisibilityBundle::default(),
				String3dSpawnRequest {
					common : CommonString3dSpawnParams {
						string : line.string.clone(),
						color : line.foreground,
						background_color: Some(line.background),
						..default()
					},
					..default()
				}
			))
			.id();

			commands.entity(lines_entity).add_child(word_entity);
		}

		lines_entity
	}

	pub fn spawn_framerate_dot(
		dot_radius		: f32,
		framerate_color	: Color,
		translation		: Vec3,
		mesh_assets		: &mut Assets<Mesh>,
		material_assets	: &mut Assets<StandardMaterial>,
		color_materials_cache : &mut ColorMaterialsCache,
		commands		: &mut Commands,
	) -> Entity {
		profile_function!();

		let mesh_handle = mesh_assets.add(shape::Circle::new(dot_radius).into());

		let material_handle = get_color_material_handle(
			framerate_color,
			color_materials_cache,
			material_assets
		);

		commands.spawn((
			FramerateIndicator::default(),
			PbrBundle {
				mesh		: mesh_handle,
				material	: material_handle.clone_weak(),
				transform	: Transform::from_translation(translation),
				..default()
			},
			RigidBody	:: Fixed,
			Collider	:: ball(dot_radius),
			RaypickHover:: default()
		)).id()
	}
}

#[derive(Component)]
pub struct FramerateIndicator {
	pub highlighted : bool,
	pub radius		: f32,
}

impl Default for FramerateIndicator {
	fn default() -> Self {
        Self {
			highlighted : false,
			radius		: 0.03,
		}
    }
}

impl FramerateIndicator {
	pub fn highlight(
		&mut self,
		entity		: Entity,
		transform	: &Transform,
		commands	: &mut Commands,
	) {
		if self.highlighted {
			return
		}
		
		let duration 		= Duration::from_millis(250);
		let ease			= EaseFunction::CubicIn;

		let scale			= 1.07;
		let hovered_scale	= transform.scale * scale;

		let tween = Tween::new(
			ease,
			duration,
			TransformLens {
				start : transform.clone(),
				end : Transform {
					translation : transform.translation,
					rotation : transform.rotation,
					scale : hovered_scale,
					..default()
				}
			}
		);

		commands.entity(entity).insert(Animator::new(tween));

		self.highlighted = true;
	}
	
	pub fn unhighlight(
		&mut self,
		entity		: Entity,
		transform	: &Transform,
		commands	: &mut Commands,
	) {
		if !self.highlighted {
			return
		}
		
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

		commands.entity(entity).insert(Animator::new(tween));

		self.highlighted = false;
	}
	
	pub fn click(
		&mut self,
		entity		: Entity,
		transform	: &Transform,
		commands	: &mut Commands,
	) {
		let duration 		= Duration::from_millis(150);
		let ease			= EaseFunction::CubicIn;

		let scale			= 1.17;
		let hovered_scale	= transform.scale * scale;

		let tween = Tween::new(
			ease,
			duration,
			TransformLens {
				start : transform.clone(),
				end : Transform {
					translation : transform.translation,
					rotation : transform.rotation,
					scale : hovered_scale,
					..default()
				}
			}
		)
		.with_repeat_count(2)
		.with_repeat_strategy(RepeatStrategy::MirroredRepeat)
		;

		commands.entity(entity).insert(Animator::new(tween));

		self.highlighted = true;
	}}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
pub struct BevyFramerateManagerSystems;

pub struct BevyFramerateManagerPlugin;

impl Plugin for BevyFramerateManagerPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(FramerateManager::default())
			.insert_resource(FramerateDebug::default())

			// animate tweens only with adequate fps
			.configure_set(
				AnimationSystem::AnimationUpdate.in_base_set(CoreSet::Update)
				.run_if(conditions::animations_allowed)
			)

			.add_systems(
				(
					systems::visualize,
					systems::update,
					systems::mouse_input,
					apply_system_buffers
				)
				.chain()
				.in_set(BevyFramerateManagerSystems)
			)
			.add_systems(
				(
					systems::animations_keepalive,
					systems::animations_cleanup_components,
				).in_base_set(CoreSet::PostUpdate)
			)
		;
	}
}

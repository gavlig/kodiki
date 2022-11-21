use bevy :: prelude :: *;
use bevy :: input :: *;
use bevy :: input :: keyboard :: *;
use bevy_text_mesh :: prelude :: * ;

use bevy_debug_text_overlay :: screen_print;

use super :: SurfaceBevy;
use super :: SurfacesMapBevy;
use super :: CursorBevy;
use super :: TextCache;
use super :: HelixColorsCache;
use super :: application :: Application;
use super :: render;
use super :: editor :: EditorViewBevy;

use crate :: game :: DespawnResource;
use crate :: game :: FontAssetHandles;

use crate :: bevy_ab_glyph :: ABGlyphFont;

use helix_term  :: config		:: Config;
use helix_term  :: args			:: Args;
use helix_term	:: compositor	:: SurfacesMap as SurfacesMapHelix;
use helix_tui   :: buffer		:: Buffer as SurfaceHelix;
use helix_view  :: graphics 	:: *;
use helix_view  :: keyboard 	:: KeyCode as KeyCodeHelix;

use anyhow      :: { Context, Error, Result };

use std :: path :: PathBuf;

fn setup_logging(logpath: PathBuf, verbosity: u64) -> Result<()> {
	let mut base_config = fern::Dispatch::new();

	base_config = match verbosity {
		0 => base_config.level(log::LevelFilter::Warn),
		1 => base_config.level(log::LevelFilter::Info),
		2 => base_config.level(log::LevelFilter::Debug),
		_3_or_more => base_config.level(log::LevelFilter::Trace),
	};

	// Separate file config so we can include year, month and day in file logs
	let file_config = fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!(
				"{} {} [{}] {}",
				chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
				record.target(),
				record.level(),
				message
			))
		})
		.chain(fern::log_file(logpath)?);

	base_config.chain(file_config).apply()?;

	Ok(())
}

pub fn startup(
	world: &mut World
) {
	let mut surfaces_helix = SurfacesMapHelix::default();
	let 	surfaces_bevy = SurfacesMapBevy::default();
	
	let rect = Rect {
		x : 0,
		y : 0,
		width : 100,
		height : 40,
	};

	let surface_editor = SurfaceHelix::empty(rect);
	surfaces_helix.insert(String::from(EditorViewBevy::ID), surface_editor);

	world.insert_resource(surfaces_helix);
	world.insert_resource(surfaces_bevy);

	let app = startup_impl(rect);
	world.insert_non_send_resource(app.unwrap());
}

#[tokio::main]
async fn startup_impl(area: Rect) -> Result<Application, Error> {
	let args = Args::parse_args().context("could not parse arguments").unwrap();

	// let logpath = args.log_file.as_ref().cloned().unwrap_or(helix_loader::log_file());
	// setup_logging(logpath, args.verbosity).context("failed to initialize logging").unwrap();

	let config_dir = helix_loader::config_dir();
	if !config_dir.exists() {
		std::fs::create_dir_all(&config_dir).ok();
	}

	helix_loader::initialize_config_file(args.config_file.clone());

	let config = match std::fs::read_to_string(helix_loader::config_file()) {
		Ok(config) => toml::from_str(&config)
			.map(helix_term::keymap::merge_keys)
			.unwrap_or_else(|err| {
				eprintln!("Bad config: {}", err);
				eprintln!("Press <ENTER> to continue with default config");
				use std::io::Read;
				let _ = std::io::stdin().read(&mut []);
				Config::default()
			}),
		Err(err) if err.kind() == std::io::ErrorKind::NotFound => Config::default(),
		Err(err) => { eprintln!("Error while loading config from {}: {}", helix_loader::config_file().display(), err); return Err(anyhow::anyhow!("!!!")); }
	};

	let app = Application::new(args, config, area).context("unable to create new application");

	app
}

pub fn render(
	mut surfaces_helix	: ResMut<SurfacesMapHelix>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
	mut fonts           : ResMut<Assets<TextMeshFont>>,
		fonts2			: Res<Assets<ABGlyphFont>>,
		font_handles    : Res<FontAssetHandles>,
	mut	cursor          : ResMut<CursorBevy>,
	mut	q_cursor_transform : Query<&mut Transform>,
		app             : Option<NonSendMut<Application>>,
		time			: Res<Time>,

	(mut ttf2_mesh_cache, mut mesh_cache, mut helix_colors_cache) 
	:
	(ResMut<TTF2MeshCache>, ResMut<TextCache>, ResMut<HelixColorsCache>),

	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut despawn         : ResMut<DespawnResource>,
	mut commands        : Commands,
) {
	if app.is_none() {
		return;
	}

	let mut app = app.unwrap();

	let editor_area = app.area;

	// erase previous frame
	for (_name, surface) in surfaces_helix.iter_mut() {
		surface.reset();
	}

	let old_style = false;

	// first let helix render into surface_helix
	if old_style {
		let surface_helix_editor = surfaces_helix.get_mut(&String::from(EditorViewBevy::ID)).unwrap();
		app.render(surface_helix_editor);
	} else {
		app.render_ext(editor_area, surfaces_helix.as_mut());
	}

	let (cursor_pos, cursor_kind) = app.cursor(editor_area);
	if let Some(cursor_pos) = cursor_pos {
		// cursor position changed so we reset easing timer
		if cursor.x != cursor_pos.0
		|| cursor.y != cursor_pos.1
		{
			cursor.easing_accum = 0.0;
		}

		cursor.x = cursor_pos.0;
		cursor.y = cursor_pos.1;
		cursor.kind = cursor_kind;
	}

	let mut surface_names_str = String::default();
	surface_names_str.push_str(format!("{} helix layers:\n", surfaces_helix.len()).as_str());
	for (name, surface) in surfaces_helix.iter() {
		surface_names_str.push_str(" - ");
		surface_names_str.push_str(format!("{} len: {}", name, surface.content.len()).as_str());
		surface_names_str.push('\n');
	}
	screen_print!("\n{}", surface_names_str);

	let font_handle = &font_handles.share_tech;
	let font		= fonts.get_mut(font_handle).unwrap();

	let font_handle2 = &font_handles.ubuntu_mono;
	let font2		= fonts2.get(font_handle2).unwrap();

	let mut pos		= Vec3::new(0.0, 0.0, 0.5);
	
	{ // clean up unused surfaces
		let mut to_remove = Vec::<String>::default();
		
		for (layer_name, surface_helix) in surfaces_helix.iter_mut() {
			if surface_helix.dirty {
				continue;
			}
					
			to_remove.push(layer_name.clone());
			println!("unused helix surface removed: {}", layer_name);
		}
	
		for layer in to_remove.iter() {
			surfaces_helix.remove(layer);
		}
		
		for (layer_name, surface_bevy) in surfaces_bevy.iter_mut() {
			if surfaces_helix.contains_key(layer_name) {
				continue;
			}
			
			despawn.entities.push(surface_bevy.entity.unwrap());
		
			to_remove.push(layer_name.clone());
			println!("unused bevy surface removed: {}", layer_name);
		}
	
		for layer in to_remove {
			surfaces_bevy.remove(&layer);
		}
	}

	// create bevy surfaces for every helix surface
	for (layer_name, surface_helix) in surfaces_helix.iter() {
		if surfaces_bevy.contains_key(layer_name) {
			continue;
		}

		let mut surface_bevy = SurfaceBevy::default();

		let layer_entity =
		super::spawn::surface(
			layer_name,
			&surface_helix,
			&mut surface_bevy,
			&mut font.ttf_font,
			pos,
			&mut mesh_cache.meshes,
			&mut helix_colors_cache.materials,
			mesh_assets.as_mut(),
			material_assets.as_mut(),
			&mut commands
		);

		surface_bevy.entity = Some(layer_entity);
		surfaces_bevy.insert(layer_name.clone(), surface_bevy);

		println!("new bevy surface created: {}", layer_name);
	}
	
	pos.z = 0.0;

	// render surfaces
	for (layer_name, surface_helix) in surfaces_helix.iter_mut() {
		let surface_bevy = surfaces_bevy.get_mut(layer_name).unwrap();

		render::surface(
			pos,
			surface_helix,
			surface_bevy,
			cursor.as_ref(),
			&mut font.ttf_font,
			&font2,
			&mut ttf2_mesh_cache,
			&mut mesh_cache.meshes,
			&mut helix_colors_cache.materials,
			&mut mesh_assets,
			&mut material_assets,
			despawn.as_mut(),
			&mut commands
		);
		
		// println!("rendering surface {} len {}", layer_name, surface_bevy.content.len());
		// println!("layer content:");
		// for y in 0..surface_helix.area.height {
		// 	for x in 0..surface_helix.area.width {
		// 		let content_index = (y * surface_helix.area.width + x) as usize;
		// 		print!("{}", surface_helix.content[content_index].symbol);
		// 	}
		// 	print!("\n");
		// }
	}

	{ // render cursor
		let surface_bevy_editor = surfaces_bevy.get(&String::from(EditorViewBevy::ID)).unwrap();
		render::cursor(
			pos,
			surface_bevy_editor,
			&mut font.ttf_font,
			cursor.as_mut(),
			&mut q_cursor_transform,
			&time,
			&app.editor.theme,
			&mut helix_colors_cache.materials,
			&mut material_assets,
			&mut mesh_assets,
			&mut commands
		);
	}
}

#[tokio::main]
pub async fn input(
	mut ev_keyboard : EventReader<KeyboardInput>,
	key			    : Res<Input<KeyCode>>,
	app             : Option<NonSendMut<Application>>,
) {
	if app.is_none() {
		return;
	}
	let mut app = app.unwrap();

	for e in ev_keyboard.iter() {
		if e.state != ButtonState::Pressed {
			continue;
		}

		if e.key_code.is_none() {
			continue;
		}

		let helix_keycode =
		match e.key_code.unwrap() {
			KeyCode::Back => KeyCodeHelix::Backspace,
			KeyCode::Return => KeyCodeHelix::Enter,
			KeyCode::Left => KeyCodeHelix::Left,
			KeyCode::Right => KeyCodeHelix::Right,
			KeyCode::Up => KeyCodeHelix::Up,
			KeyCode::Down => KeyCodeHelix::Down,
			KeyCode::Home => KeyCodeHelix::Home,
			KeyCode::End => KeyCodeHelix::End,
			KeyCode::PageUp => KeyCodeHelix::PageUp,
			KeyCode::PageDown => KeyCodeHelix::PageDown,
			KeyCode::Tab => KeyCodeHelix::Tab,
			KeyCode::Delete => KeyCodeHelix::Delete,
			KeyCode::Insert => KeyCodeHelix::Insert,
			KeyCode::Escape => KeyCodeHelix::Esc,

			KeyCode::Space => KeyCodeHelix::Char(' '),
			KeyCode::Underline => KeyCodeHelix::Char('_'),

			KeyCode::Key0 => KeyCodeHelix::Char('0'),
			KeyCode::Key1 => KeyCodeHelix::Char('1'),
			KeyCode::Key2 => KeyCodeHelix::Char('2'),
			KeyCode::Key3 => KeyCodeHelix::Char('3'),
			KeyCode::Key4 => KeyCodeHelix::Char('4'),
			KeyCode::Key5 => KeyCodeHelix::Char('5'),
			KeyCode::Key6 => KeyCodeHelix::Char('6'),
			KeyCode::Key7 => KeyCodeHelix::Char('7'),
			KeyCode::Key8 => KeyCodeHelix::Char('8'),
			KeyCode::Key9 => KeyCodeHelix::Char('9'),

			KeyCode::Q => KeyCodeHelix::Char('q'),
			KeyCode::W => KeyCodeHelix::Char('w'),
			KeyCode::E => KeyCodeHelix::Char('e'),
			KeyCode::R => KeyCodeHelix::Char('r'),
			KeyCode::T => KeyCodeHelix::Char('t'),
			KeyCode::Y => KeyCodeHelix::Char('y'),

			KeyCode::U => KeyCodeHelix::Char('u'),
			KeyCode::I => KeyCodeHelix::Char('i'),
			KeyCode::O => KeyCodeHelix::Char('o'),
			KeyCode::P => KeyCodeHelix::Char('p'),
			KeyCode::LBracket => KeyCodeHelix::Char('['),
			KeyCode::RBracket => KeyCodeHelix::Char(']'),
			KeyCode::Backslash => KeyCodeHelix::Char('\\'),

			KeyCode::A => KeyCodeHelix::Char('a'),
			KeyCode::S => KeyCodeHelix::Char('s'),
			KeyCode::D => KeyCodeHelix::Char('d'),
			KeyCode::F => KeyCodeHelix::Char('f'),
			KeyCode::G => KeyCodeHelix::Char('g'),

			KeyCode::H => KeyCodeHelix::Char('h'),
			KeyCode::J => KeyCodeHelix::Char('j'),
			KeyCode::K => KeyCodeHelix::Char('k'),
			KeyCode::L => KeyCodeHelix::Char('l'),
			KeyCode::Semicolon => KeyCodeHelix::Char(';'),
			KeyCode::Colon => KeyCodeHelix::Char(':'),
			KeyCode::Apostrophe => KeyCodeHelix::Char('\''),

			KeyCode::Z => KeyCodeHelix::Char('z'),
			KeyCode::X => KeyCodeHelix::Char('x'),
			KeyCode::C => KeyCodeHelix::Char('c'),
			KeyCode::V => KeyCodeHelix::Char('v'),
			KeyCode::B => KeyCodeHelix::Char('b'),

			KeyCode::N => KeyCodeHelix::Char('n'),
			KeyCode::M => KeyCodeHelix::Char('m'),
			KeyCode::Comma => KeyCodeHelix::Char(','),
			KeyCode::Convert => KeyCodeHelix::Char('.'),
			KeyCode::Slash => KeyCodeHelix::Char('/'),
			_ => { println!("skipping keycode {:?}", e.key_code); continue; }
		};

		let mut modifiers = helix_view::keyboard::KeyModifiers::NONE;

		if key.pressed(KeyCode::LAlt) || key.pressed(KeyCode::RAlt) {
			modifiers.insert(helix_view::keyboard::KeyModifiers::ALT);
		}

		if key.pressed(KeyCode::LControl) || key.pressed(KeyCode::RControl) {
			modifiers.insert(helix_view::keyboard::KeyModifiers::CONTROL);
		}

		if key.pressed(KeyCode::LShift) || key.pressed(KeyCode::RShift) {
			modifiers.insert(helix_view::keyboard::KeyModifiers::SHIFT);
		}

		let key_event = helix_view::input::KeyEvent {
			code : helix_keycode,
			modifiers : modifiers,
		};

		let event = helix_view::input::Event::Key(key_event);
		app.handle_event(&event);
	}
}
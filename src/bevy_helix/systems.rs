use bevy :: prelude :: *;
use bevy :: input :: *;
use bevy :: input :: keyboard :: *;
use bevy_text_mesh :: prelude :: * ;

use super :: BevyHelix;
use super :: SurfaceBevy;
use super :: application :: Application;
use super :: render;

use crate :: game :: DespawnResource;
use crate :: game :: FontAssetHandles;

use helix_term  :: config :: Config;
use helix_term  :: args :: Args;
use helix_tui   :: buffer :: Buffer as SurfaceHelix;
use helix_view  :: graphics :: *;
use helix_view  :: keyboard :: KeyCode as KeyCodeHelix;

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
    let rect = Rect {
        x : 0,
        y : 0,
        width : 160,
        height : 40,
    };

    let surface = SurfaceHelix::empty(rect);
    world.insert_resource(surface);

    let surface_bevy = SurfaceBevy::empty(rect);
    world.insert_resource(surface_bevy);

    let app = startup_impl();
    world.insert_non_send_resource(app.unwrap());
}

#[tokio::main]
async fn startup_impl() -> Result<Application, Error> {
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

    let app = Application::new(args, config).context("unable to create new application");

    app
}

pub fn render(
    mut surface_helix   : ResMut<SurfaceHelix>,
    mut surface_bevy    : ResMut<SurfaceBevy>,
    mut fonts           : ResMut<Assets<TextMeshFont>>,
        font_handles    : Res<FontAssetHandles>,
        q_bevy_helix    : Query<Entity, With<BevyHelix>>,
        app             : Option<NonSendMut<Application>>,
    mut despawn         : ResMut<DespawnResource>,
    mut commands        : Commands,
) {
    if app.is_none() {
        return;
    }
    let mut app = app.unwrap();

    // erase previous frame
    surface_helix.reset();

    app.render(surface_helix.as_mut());

    let font_handle = &font_handles.share_tech;
	let font		= fonts.get_mut(font_handle).unwrap();

    for bevy_helix_entity in q_bevy_helix.iter() {
        render::surface(
            bevy_helix_entity,
            surface_helix.as_mut(),
            surface_bevy.as_mut(),
            font_handle,
            &mut font.ttf_font,
            despawn.as_mut(), 
            &mut commands
        );
    }
}

pub fn input(
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

        // KeyEventKind

        let event = helix_view::input::Event::Key(key_event);
        app.handle_event(&event);
        // let compositor = Box::new(&app.compositor) as Box<dyn helix_term::compositor::Compositor>;
        println!("Keyboard event! bevy {:?} helix {:?}", e, event);
    }
}
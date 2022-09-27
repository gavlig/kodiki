use bevy :: prelude :: *;

use super :: application :: Application;
use super :: SurfaceBevy;

use helix_term  :: config :: Config;
use helix_term  :: args :: Args;
use helix_tui   :: buffer :: Buffer as Surface;
use helix_view  :: graphics :: *;

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
        width : 200,
        height : 80,
    };

    let surface = Surface::empty(rect);
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
    mut surface : ResMut<Surface>,
    app : Option<NonSendMut<Application>>,
) {
    if app.is_none() {
        return;
    }
    let mut app = app.unwrap();
    
    app.render(surface.as_mut());

    // render_tui_surface(surface.as_ref());
}
use bevy :: prelude :: *;

use super :: application :: Application;

use helix_term :: config :: Config;
use helix_tui :: buffer :: Buffer as Surface;

pub fn statup(
    commands : Commands
) {
    let logpath = args.log_file.as_ref().cloned().unwrap_or(logpath);
    setup_logging(logpath, args.verbosity).context("failed to initialize logging")?;

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
        Err(err) => return Err(Error::new(err)),
    };

    let mut app = Application::new(args, config).context("unable to create new application")?;

    let app = Application::new(args, config);
    commands.insert_resource(app);
}

pub fn render(
    surface : Res<Surface>,
    app : Option<Res<Application>>,
) {
    if app.is_none() {
        return;
    }
    
    app.render(surface);
}
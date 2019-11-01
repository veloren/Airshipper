use crate::config::ClientConfig;
use crate::{logger, Result};
use clap::{load_yaml, App};

pub fn run(config: &mut ClientConfig) -> Result<()> {
    let yml = load_yaml!("clap.yml");
    let version = format!("provided by airshipper v{}", env!("CARGO_PKG_VERSION"));
    let app = App::from_yaml(yml).version(&*version);
    let m = app.clone().get_matches();

    let level = match m.occurrences_of("log") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    };

    if let Err(e) = logger::log(&config, level) {
        panic!("Failed to set logging: {}", e);
    }

    log::debug!("Running on {}", whoami::os());

    if m.is_present("gui") {
        // Exit early for gui to take over.
        return Ok(());
    }
    // handle arguments otherwise
    if m.is_present("update") {
        log::info!("Updating...");
        config.update()?;
    } else if m.is_present("start") {
        log::info!("Starting...");
        config.start()?;
    } else {
        // Default to checking for updates and starting the game.
        log::info!("Checking for updates...");
        config.update()?;
        log::info!("Starting...");
        config.start()?;
    }

    // Exit so gui won't start
    std::process::exit(0);
}

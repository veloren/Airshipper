use crate::{logger, filesystem, gui, Result};
use clap::{load_yaml, App};

/// Process command line arguments and start GUI
pub fn process() -> Result<()> {
    let yml = load_yaml!("clap.yml");
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let app = App::from_yaml(yml).version(&*version);
    let m = app.clone().get_matches();

    let level = match m.occurrences_of("log") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    };

    if let Err(e) = logger::log(level) {
        panic!("Failed to set logging: {}", e);
    }

    log::debug!("Running on {}", whoami::os());
    log::debug!("Base Path: {}", filesystem::base_path());
    log::debug!("Log file Path: {}", filesystem::get_log_path().display());
    
    // handle arguments otherwise
    if m.is_present("update") {
        log::info!("Updating...");
        // TODO: Update only, no GUI
    } else if m.is_present("start") {
        log::info!("Starting...");
        // TODO: Start only, no GUI
    } else if m.is_present("run"){
        // Default to checking for updates and starting the game.
        log::info!("Checking for updates...");
        // TODO: Update only, no GUI
        log::info!("Starting...");
        // TODO: Start only, no GUI
    } else {
        gui::run();
    }
    Ok(())
}

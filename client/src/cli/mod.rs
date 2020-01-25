use crate::{filesystem, gui, logger, /*state::State, profiles::Profile,*/ Result};
use clap::{load_yaml, App};

/// Process command line arguments and optionally start GUI
pub async fn process() -> Result<()> {
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

    //let state = State::load().await?;

    // handle arguments
    if m.is_present("update") {
        //update(&mut state).await?;
    } else if m.is_present("start") {
        log::info!("Starting...");
    //start(&mut state, &state.active_profile).await?;
    } else if m.is_present("run") {
        //update(&mut state).await?;
        //start(&mut state, &state.active_profile).await?;
    } else {
        gui::run();
    }
    Ok(())
}

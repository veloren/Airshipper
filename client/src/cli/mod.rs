#[cfg(feature = "gui")]
use crate::gui;
use crate::{filesystem, logger, state::SavedState, Result};
use clap::{load_yaml, App};

/// Process command line arguments and optionally starts GUI
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

    log::debug!("Running on {}", std::env::consts::OS);
    log::debug!("Base Path: {}", filesystem::base_path());
    log::debug!("Log file: {}", filesystem::get_log_path().display());
    log::debug!("Assets Path: {}", filesystem::assets_path());
    #[cfg(windows)]
    log::debug!("Cache Path: {}", filesystem::get_cache_path().display());

    // Check for updates (windows only)
    #[cfg(windows)]
    crate::updater::update().await?;

    let mut state = SavedState::load().await.unwrap_or_default();

    // handle arguments
    process_arguments(&mut state, m).await?;

    // Save state
    state.save().await?;

    Ok(())
}

async fn process_arguments<'n, 'a>(
    mut state: &mut SavedState,
    m: clap::ArgMatches<'n>,
) -> Result<()> {
    if m.is_present("update") {
        update(&mut state, true).await?;
    } else if m.is_present("start") {
        // TODO: Check if profile is installed...
        log::info!("Starting...");
        start(&mut state).await?;
    } else if m.is_present("run") {
        update(&mut state, false).await?;
        start(&mut state).await?;
    } else {
        #[cfg(feature = "gui")]
        gui::run();
        #[cfg(not(feature = "gui"))]
        {
            update(&mut state, false).await?;
            start(&mut state).await?;
        }
    }
    Ok(())
}

async fn update(state: &mut SavedState, do_not_ask: bool) -> Result<()> {
    if state.check_for_profile_update().await? != state.active_profile.version {
        if do_not_ask {
            log::info!("Updating...");
            let metrics = state.update_profile().await?;
            print_progress(metrics).await;
            log::info!("Extracting...");
            state.install_profile().await?;
            log::info!("Done!");
        } else {
            log::info!("Update found, do you want to update? [Y/n]");
            if confirm_action()? {
                let metrics = state.update_profile().await?;
                print_progress(metrics).await;
                log::info!("Extracting...");
                state.install_profile().await?;
                log::info!("Done!");
            }
        }
    } else {
        log::info!("Profile already up-to-date.");
    }
    Ok(())
}

async fn print_progress(metrics: isahc::Metrics) {
    use indicatif::{FormattedDuration, HumanBytes, ProgressBar, ProgressStyle};

    let bar = ProgressBar::new(0).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.green/white}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("=>-"),
    );

    loop {
        let percentage =
            ((metrics.download_progress().0 * 100) / metrics.download_progress().1) as f32;
        if percentage >= 100.0 {
            break;
        }
        bar.set_position(metrics.download_progress().0);
        bar.set_length(metrics.download_progress().1);
        bar.set_message(&format!(
            "time: {}  speed: {}/sec",
            FormattedDuration(metrics.total_time()),
            HumanBytes(metrics.download_speed() as u64),
        ));

        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

async fn start(state: &mut SavedState) -> Result<()> {
    log::info!("Starting...");
    Ok(state.start_profile().await?)
}

/// Will read from stdin for confirmation
/// NOTE: no input = true
/// Temporary...
pub fn confirm_action() -> Result<bool> {
    let mut buffer = String::new();
    let _ = std::io::stdin().read_line(&mut buffer)?;
    buffer = buffer.to_lowercase();

    if buffer.trim().is_empty() {
        Ok(true)
    } else if buffer.starts_with("y") {
        Ok(true)
    } else if buffer.starts_with("n") {
        Ok(false)
    } else {
        // for the accidental key smash
        Ok(false)
    }
}

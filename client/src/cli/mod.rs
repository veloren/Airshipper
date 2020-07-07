use crate::{filesystem, gui, logger, state::SavedState, Result};
use parse::Action;
mod parse;

pub use parse::CmdLine;

/// Process command line arguments and optionally starts GUI
pub async fn process() -> Result<()> {
    let cmd = CmdLine::new();

    let level = match cmd.debug {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    };

    logger::log(level);

    log::debug!("Running on {}", std::env::consts::OS);
    log::debug!("Base Path: {}", filesystem::base_path());
    log::debug!("Log file: {}", filesystem::get_log_path().display());
    #[cfg(windows)]
    log::debug!("Cache Path: {}", filesystem::get_cache_path().display());
    log::debug!("Cmdline args: {:?}", cmd);

    // Check for updates (windows only)
    #[cfg(windows)]
    crate::updater::update().await?;

    let mut state = SavedState::load().await.unwrap_or_default();

    // handle arguments
    process_arguments(&mut state, cmd).await?;

    // Save state
    state.save().await?;

    Ok(())
}

async fn process_arguments(mut state: &mut SavedState, cmd: CmdLine) -> Result<()> {
    match cmd.action {
        // CLI
        Some(action) => match action {
            Action::Update => update(&mut state, true).await?,
            Action::Start => start(&mut state).await?,
            Action::Run => {
                update(&mut state, false).await?;
                start(&mut state).await?
            },
        },
        // GUI
        None => gui::run(cmd),
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

    let progress_bar = ProgressBar::new(0).with_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] [{bar:40.green/white}] {bytes}/{total_bytes} \
                 ({eta})",
            )
            .progress_chars("=>-"),
    );

    loop {
        let percentage = ((metrics.download_progress().0 * 100)
            / metrics.download_progress().1) as f32;
        if percentage >= 100.0 {
            break;
        }
        progress_bar.set_position(metrics.download_progress().0);
        progress_bar.set_length(metrics.download_progress().1);
        progress_bar.set_message(&format!(
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

    if buffer.trim().is_empty() || buffer.starts_with('y') {
        return Ok(true);
    } else if buffer.starts_with('n') {
        return Ok(false);
    }
    // for the accidental key smash
    Ok(false)
}

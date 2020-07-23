use crate::{fs, gui, io, logger, net, profiles::Profile, state::SavedState, Result};
use parse::Action;
mod parse;
use iced::futures::stream::StreamExt;

pub use parse::CmdLine;

/// Process command line arguments and optionally starts GUI
pub fn process() -> Result<()> {
    let cmd = CmdLine::new();

    let level = match cmd.debug {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    };

    logger::log(level);

    log::debug!("Running on {}", std::env::consts::OS);
    log::debug!("Base Path: {}", fs::base_path());
    log::debug!("Log file: {}", fs::log_file().display());
    #[cfg(windows)]
    log::debug!("Cache Path: {}", fs::get_cache_path().display());
    log::debug!("Cmdline args: {:?}", cmd);

    // TODO: Iced does not allow us to create the global async runtime ourself :/
    let mut rt = tokio::runtime::Runtime::new()?;

    // We ignore any errors to avoid disrupting playing the game.
    #[cfg(windows)]
    let _ = rt.block_on(crate::windows::update());

    if cmd.action.is_some() {
        if let Err(e) = rt.block_on(async {
            let mut state = SavedState::load().await.unwrap_or_default();

            // handle arguments
            process_arguments(&mut state, cmd).await?;

            // Save state
            state.save().await?;

            Ok(())
        }) {
            return Err(e);
        }
    } else {
        rt.shutdown_timeout(std::time::Duration::from_millis(500));
        gui::run(cmd);
    }
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
    if let Some(version) = Profile::update(state.active_profile.clone()).await? {
        if do_not_ask {
            log::info!("Updating...");
            download(state.active_profile.clone()).await?;
            log::info!("Extracting...");
            state.active_profile =
                Profile::install(state.active_profile.clone(), version).await?;
            log::info!("Done!");
        } else {
            log::info!("Update found, do you want to update? [Y/n]");
            if confirm_action()? {
                log::info!("Updating...");
                download(state.active_profile.clone()).await?;
                log::info!("Extracting...");
                state.active_profile =
                    Profile::install(state.active_profile.clone(), version).await?;
                log::info!("Done!");
            }
        }
    } else {
        log::info!("Profile already up-to-date.");
    }
    Ok(())
}

async fn download(profile: Profile) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    let progress_bar = ProgressBar::new(0).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.green/white}] {msg} [{eta}]")
            .progress_chars("=>-"),
    );
    progress_bar.set_length(100);

    let mut stream = crate::net::download(profile.url(), profile.download_path()).boxed();

    while let Some(progress) = stream.next().await {
        match progress {
            net::Progress::Started => {},
            net::Progress::Errored(e) => return Err(e.into()),
            net::Progress::Finished => return Ok(()),
            net::Progress::Advanced(msg, percentage) => {
                progress_bar.set_position(percentage);
                progress_bar.set_message(&msg);
            },
        }
    }
    Ok(())
}

async fn start(state: &mut SavedState) -> Result<()> {
    log::info!("Starting...");
    let mut stream =
        crate::io::stream_process(Profile::start(state.active_profile.clone())).boxed();

    while let Some(progress) = stream.next().await {
        match progress {
            io::ProcessUpdate::Line(line) => log::info!("[Veloren] {}", line),
            io::ProcessUpdate::Exit(exit) => log::info!("Veloren exited with {}", exit),
            io::ProcessUpdate::Error(e) => return Err(e.into()),
        }
    }
    Ok(())
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
    Ok(false)
}

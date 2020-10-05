use crate::{fs, gui, io, logger, net, profiles::Profile, Result};
use parse::Action;
mod parse;
use iced::futures::stream::StreamExt;

use gui::Airshipper;
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

    if cmd.action.is_some() {
        // let the user know incase airshipper can be updated.
        #[cfg(windows)]
        if let Ok(Some(release)) = crate::windows::query() {
            log::info!(
                "New Airshipper release found: {}. Run `airshipper upgrade` to update.",
                release.version
            );
        }

        if let Err(e) = rt.block_on(async {
            let mut state = Airshipper::load(cmd.clone()).await;

            // handle arguments
            process_arguments(&mut state.active_profile, cmd).await?;

            // Save state
            state.save_mut().await?;

            Ok(())
        }) {
            return Err(e);
        }
    } else {
        rt.shutdown_timeout(std::time::Duration::from_millis(500));
        gui::run(cmd)?;
    }
    Ok(())
}

async fn process_arguments(mut profile: &mut Profile, cmd: CmdLine) -> Result<()> {
    match cmd.action {
        // CLI
        Some(action) => match action {
            Action::Update => update(&mut profile, true).await?,
            Action::Start => start(&mut profile, cmd.verbose).await?,
            Action::Run => {
                update(&mut profile, false).await?;
                start(&mut profile, cmd.verbose).await?
            },
            #[cfg(windows)]
            Action::Upgrade => {
                tokio::task::block_in_place(upgrade)?;
            },
        },
        // GUI
        None => gui::run(cmd)?,
    }
    Ok(())
}

async fn update(profile: &mut Profile, do_not_ask: bool) -> Result<()> {
    if let Some(version) = Profile::update(profile.clone()).await? {
        if do_not_ask {
            log::info!("Updating...");
            download(profile.clone()).await?;
            log::info!("Extracting...");
            *profile = Profile::install(profile.clone(), version).await?;
            log::info!("Done!");
        } else {
            log::info!("Update found, do you want to update? [Y/n]");
            if confirm_action()? {
                log::info!("Updating...");
                download(profile.clone()).await?;
                log::info!("Extracting...");
                *profile = Profile::install(profile.clone(), version).await?;
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

async fn start(profile: &mut Profile, verbosity: i32) -> Result<()> {
    if !profile.installed() {
        log::info!("Profile is not installed. Install it via `airshipper update`");
        return Ok(());
    }

    log::info!("Starting...");
    let mut stream =
        crate::io::stream_process(&mut Profile::start(profile, verbosity))?.boxed();

    while let Some(progress) = stream.next().await {
        match progress {
            io::ProcessUpdate::Line(line) => log::info!("[Veloren] {}", line),
            io::ProcessUpdate::Exit(exit) => log::info!("Veloren exited with {}", exit),
            io::ProcessUpdate::Error(e) => return Err(e.into()),
        }
    }
    Ok(())
}

#[cfg(windows)]
fn upgrade() -> Result<()> {
    match crate::windows::query()? {
        Some(release) => {
            log::info!("Found new Airshipper release: {}", release.version);
            crate::windows::update(&release)?;
        },
        None => log::info!("Airshipper is up-to-date."),
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

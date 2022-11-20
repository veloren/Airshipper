use crate::{
    fs, gui, io, logger, net,
    profiles::{parse_env_vars, Profile, WGPU_BACKENDS},
    Result,
};
use parse::Action;
mod parse;
use iced::futures::stream::StreamExt;

use crate::{profiles::LogLevel, BASE_PATH};
use gui::Airshipper;
pub use parse::CmdLine;
use tracing::level_filters::LevelFilter;

/// Process command line arguments and optionally starts GUI
pub fn process() -> Result<()> {
    let mut cmd = CmdLine::new();

    let level = match cmd.debug {
        0 => LevelFilter::INFO,
        1 => LevelFilter::DEBUG,
        2 => LevelFilter::TRACE,
        _ => LevelFilter::TRACE,
    };

    let log = fs::log_path_file();
    let _guard = logger::init(Some((log.0, log.1)), level);

    tracing::debug!("Running on {}", std::env::consts::OS);
    tracing::debug!("Base Path: {}", fs::base_path());
    tracing::debug!("Log file: {}", fs::log_file().display());
    #[cfg(windows)]
    tracing::debug!("Cache Path: {}", fs::get_cache_path().display());
    tracing::debug!("Cmdline args: {:?}", cmd);
    tracing::info!("Visit https://book.veloren.net/ for an FAQ and Troubleshooting");

    if cmd.force_reset {
        std::fs::remove_dir_all(BASE_PATH.as_path())?;
    }

    // GUI
    if cmd.action.is_none() {
        match gui::run(cmd.clone()) {
            Ok(_) => return Ok(()),
            Err(_) => {
                tracing::error!("Failed to start GUI. Falling back to terminal...");
                cmd.action = Some(Action::Run);
            },
        }
    }

    // CLI
    let rt = tokio::runtime::Runtime::new()?;

    // let the user know incase airshipper can be updated.
    #[cfg(windows)]
    if let Ok(Some(release)) = crate::windows::query() {
        tracing::info!(
            "New Airshipper release found: {}. Run `airshipper upgrade` to update.",
            release.version
        );
    }

    if let Err(e) = rt.block_on(async {
        let mut state = Airshipper::load(cmd.clone()).await;

        // handle arguments
        process_arguments(&mut state.active_profile, cmd.action.unwrap(), cmd.verbose)
            .await?;

        // Save state
        state.save_mut().await?;

        Ok(())
    }) {
        return Err(e);
    }
    Ok(())
}

async fn process_arguments(
    profile: &mut Profile,
    action: Action,
    verbose: i32,
) -> Result<()> {
    profile.log_level = match verbose {
        0 => LogLevel::Default,
        1 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };

    match action {
        Action::Update => update(profile, true).await?,
        Action::Start => start(profile, None).await?,
        Action::Run => {
            if let Err(e) = update(profile, false).await {
                tracing::error!(
                    ?e,
                    "Couldn't update the game, starting installed version."
                );
            }
            start(profile, None).await?
        },
        Action::Config => config(profile).await?,
        #[cfg(windows)]
        Action::Upgrade => {
            tokio::task::block_in_place(upgrade)?;
        },
    }
    Ok(())
}

async fn update(profile: &mut Profile, do_not_ask: bool) -> Result<()> {
    if let Some(version) = Profile::update(profile.clone()).await? {
        if do_not_ask {
            tracing::info!("Updating...");
            download(profile.clone()).await?;
            tracing::info!("Extracting...");
            *profile = Profile::install(profile.clone(), version).await?;
            tracing::info!("Done!");
        } else {
            tracing::info!("Update found, do you want to update? [Y/n]");
            if confirm_action()? {
                tracing::info!("Updating...");
                download(profile.clone()).await?;
                tracing::info!("Extracting...");
                *profile = Profile::install(profile.clone(), version).await?;
                tracing::info!("Done!");
            }
        }
    } else {
        tracing::info!("Profile already up-to-date.");
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
            net::Progress::Advanced(progress_data) => {
                progress_bar.set_position(progress_data.percent_complete as u64);
                progress_bar.set_message(format!(
                    "{} MB / {} MB",
                    progress_data.downloaded_bytes / 1_000_000,
                    progress_data.total_bytes / 1_000_000
                ));
            },
        }
    }
    Ok(())
}

async fn start(profile: &mut Profile, game_server_address: Option<String>) -> Result<()> {
    if !profile.installed() {
        tracing::info!("Profile is not installed. Install it via `airshipper update`");
        return Ok(());
    }

    tracing::info!("Starting...");
    let mut stream = crate::io::stream_process(&mut Profile::start(
        profile,
        game_server_address.as_deref(),
    ))?
    .boxed();

    while let Some(progress) = stream.next().await {
        match progress {
            io::ProcessUpdate::Line(line) => tracing::info!("[Veloren] {}", line),
            io::ProcessUpdate::Exit(exit) => {
                tracing::info!("Veloren exited with {}", exit)
            },
            io::ProcessUpdate::Error(e) => return Err(e.into()),
        }
    }
    Ok(())
}

async fn config(profile: &mut Profile) -> Result<()> {
    use colored::Colorize;

    let mut editor = rustyline::Editor::<()>::new()?;

    'main: loop {
        println!("===== Current configuration =====");
        let options = [
            ("Environment variables", profile.env_vars.to_string()),
            ("Graphics backend", profile.wgpu_backend.to_string()),
        ];
        for (idx, (k, v)) in options.iter().enumerate() {
            println!("- ({}) {k} = {v}", (idx + 1).to_string().blue());
        }
        println!("Which setting do you want to change? (use 'q' to quit)");

        loop {
            match editor
                .readline(&format!("{} > ", format!("1-{}", options.len()).blue()))?
                .trim()
            {
                "1" => {
                    println!(
                        "What should the environment variables be? (use 'q' to quit)"
                    );
                    println!(
                        "{}",
                        "Hint: Environment variables should be defined as key-value \
                         pairs, separated by commands.\nExample: FOO=BAR,BAZ=BIZ"
                            .dimmed()
                    );
                    loop {
                        let input = editor
                            .readline_with_initial("> ", (&profile.env_vars, ""))?;
                        if input.trim() == "q" {
                            break;
                        } else {
                            let (_, errs) = parse_env_vars(&input);
                            if !errs.is_empty() {
                                println!(
                                    "{}: Invalid environment variables:",
                                    "ERROR".red()
                                );
                                for e in errs {
                                    println!("- {e}");
                                }
                            } else {
                                profile.env_vars = input.clone();
                                println!(
                                    "{}: Environment variables have been set to \
                                     '{input}'.",
                                    "OK".green()
                                );
                                continue 'main;
                            }
                        }
                    }
                },
                "2" => {
                    println!(
                        "Which graphics backend do you want to use? (use 'q' to quit)"
                    );
                    for (idx, backend) in WGPU_BACKENDS.iter().enumerate() {
                        println!("- ({}) {}", (idx + 1).to_string().blue(), backend);
                    }
                    loop {
                        let input = editor.readline(&format!(
                            "{} > ",
                            format!("1-{}", WGPU_BACKENDS.len()).blue()
                        ))?;
                        if input.trim() == "q" {
                            break;
                        } else if let Some(backend) = input
                            .parse::<usize>()
                            .ok()
                            .and_then(|n| n.checked_sub(1))
                            .and_then(|idx| WGPU_BACKENDS.get(idx))
                        {
                            profile.wgpu_backend = *backend;
                            println!(
                                "{}: The graphics backend has been set to '{backend}'.",
                                "OK".green()
                            );
                            continue 'main;
                        } else {
                            println!("{}: Invalid option '{input}'", "ERROR".red());
                        }
                    }
                },
                "q" => break 'main Ok(()),
                input => println!("{}: Invalid option '{input}'.", "ERROR".red()),
            }
        }
    }
}

#[cfg(windows)]
fn upgrade() -> Result<()> {
    match crate::windows::query()? {
        Some(release) => {
            tracing::info!("Found new Airshipper release: {}", release.version);
            crate::windows::update(&release)?;
        },
        None => tracing::info!("Airshipper is up-to-date."),
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

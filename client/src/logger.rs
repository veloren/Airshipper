use std::path::Path;
use termcolor::{ColorChoice, StandardStream};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::{targets::Targets, LevelFilter},
    fmt::writer::MakeWriter,
    prelude::*,
    registry, EnvFilter,
};

const MAX_LOG_LINES: usize = 10_000;
const RUST_LOG_ENV: &str = "RUST_LOG";

pub fn init(log_path_file: Option<(&Path, &str)>, level: LevelFilter) -> Vec<impl Drop> {
    let mut guards: Vec<WorkerGuard> = Vec::new();
    let terminal = || StandardStream::stdout(ColorChoice::Auto);

    let base_exceptions = |env: EnvFilter| {
        env.add_directive("html5ever=error".parse().unwrap())
            .add_directive("winit=error".parse().unwrap())
            .add_directive("wgpu_native=info".parse().unwrap())
            .add_directive("strip_markdown=warn".parse().unwrap())
            .add_directive("tokio_reactor=warn".parse().unwrap())
            .add_directive("h2=info".parse().unwrap())
            .add_directive("hyper=warn".parse().unwrap())
            .add_directive("iced_wgpu::renderer=warn".parse().unwrap())
            .add_directive("iced_winit=info".parse().unwrap())
            .add_directive("iced_wgpu::image::atlas=warn".parse().unwrap())
            .add_directive("wgpu_core=warn".parse().unwrap())
            .add_directive("wgpu=warn".parse().unwrap())
            .add_directive("iced_wgpu::backend=warn".parse().unwrap())
            .add_directive("reqwest=info".parse().unwrap())
            .add_directive("gpu_alloc=warn".parse().unwrap())
            .add_directive("naga=info".parse().unwrap())
            .add_directive("rustls=info".parse().unwrap())
            .add_directive("want=info".parse().unwrap())
            .add_directive(level.into())
    };

    let filter = match std::env::var_os(RUST_LOG_ENV).map(|s| s.into_string()) {
        Some(Ok(env)) => {
            let mut filter = base_exceptions(EnvFilter::new(""));
            for s in env.split(',').into_iter() {
                match s.parse() {
                    Ok(d) => filter = filter.add_directive(d),
                    Err(err) => println!("WARN ignoring log directive: `{}`: {}", s, err),
                };
            }
            filter
        },
        _ => base_exceptions(EnvFilter::from_env(RUST_LOG_ENV)),
    };

    let registry = registry();
    let mut file_setup = false;

    let registry = {
        let (non_blocking, stdio_guard) =
            tracing_appender::non_blocking(terminal.make_writer());
        guards.push(stdio_guard);
        registry.with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
    };

    if let Some((path, file)) = log_path_file {
        // Clean up log file if possible
        let logfile = path.join(file);
        if logfile.exists() {
            if let Ok(count) =
                std::fs::read_to_string(&logfile).map(|x| x.lines().count())
            {
                if count > MAX_LOG_LINES {
                    let _ = std::fs::remove_file(&logfile);
                }
            }
        }

        match std::fs::create_dir_all(path) {
            Ok(_) => {
                let file_appender = tracing_appender::rolling::never(path, file); // It is actually rolling daily since the log name is changing daily
                let (non_blocking_file, file_guard) =
                    tracing_appender::non_blocking(file_appender);
                guards.push(file_guard);
                file_setup = true;
                let filter = filter.add_directive("output::Veloren=off".parse().unwrap());
                registry
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_writer(non_blocking_file)
                            .with_filter(
                                Targets::new()
                                    .with_default(level)
                                    .with_target("voxygen", LevelFilter::OFF),
                            ),
                    )
                    .with(filter)
                    .init();
            },
            Err(e) => {
                tracing::error!(
                    ?e,
                    "Failed to create log file!. Falling back to terminal logging only.",
                );
                registry.with(filter).init();
            },
        }
    } else {
        registry.with(filter).init();
    }

    if file_setup {
        let (path, file) = log_path_file.unwrap();
        info!(?path, ?file, "Setup terminal and file logging.");
    }

    if tracing::level_enabled!(tracing::Level::TRACE) {
        info!("Tracing Level: TRACE");
    } else if tracing::level_enabled!(tracing::Level::DEBUG) {
        info!("Tracing Level: DEBUG");
    };

    // Return the guards
    guards
}

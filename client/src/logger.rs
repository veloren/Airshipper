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

    let mut filter = EnvFilter::default().add_directive(level.into());

    let default_directives = [
        "html5ever=error",
        "winit=error",
        "wgpu_native=info",
        "strip_markdown=warn",
        "tokio_reactor=warn",
        "h2=info",
        "hyper=warn",
        "iced_wgpu::renderer=warn",
        "iced_winit=info",
        "iced_wgpu::image::atlas=warn",
        "wgpu_core=warn",
        "wgpu=warn",
        "iced_wgpu::backend=warn",
        "reqwest=info",
        "gpu_alloc=warn",
        "naga=info",
        "rustls=info",
        "want=info",
        "tokio_util::codec=error",
        "trust_dns_resolver=info",
        "trust_dns_proto=info",
    ];

    for s in default_directives {
        filter = filter.add_directive(s.parse().unwrap());
    }

    match std::env::var(RUST_LOG_ENV) {
        Ok(env) => {
            for s in env.split(',') {
                match s.parse() {
                    Ok(d) => filter = filter.add_directive(d),
                    Err(err) => eprintln!("WARN ignoring log directive: `{s}`: {err}"),
                }
            }
        },
        Err(std::env::VarError::NotUnicode(os_string)) => {
            eprintln!(
                "WARN ignoring log directives due to non-unicode data: {os_string:?}"
            );
        },
        Err(std::env::VarError::NotPresent) => {},
    };

    let filter = filter; // mutation is done

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
                let file_appender = tracing_appender::rolling::never(path, file);
                let (non_blocking_file, file_guard) =
                    tracing_appender::non_blocking(file_appender);
                guards.push(file_guard);
                file_setup = true;
                registry
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_ansi(false)
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

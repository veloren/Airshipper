use termcolor::{ColorChoice, StandardStream};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::LevelFilter, fmt::writer::MakeWriter, prelude::*, registry, EnvFilter,
};

const RUST_LOG_ENV: &str = "RUST_LOG";

pub fn init(level: LevelFilter) -> Vec<impl Drop> {
    let mut guards: Vec<WorkerGuard> = Vec::new();
    let terminal = || StandardStream::stdout(ColorChoice::Auto);

    let mut filter = EnvFilter::default().add_directive(level.into());

    let default_directives = [
        "tokio_reactor=warn",
        "h2=info",
        "hyper=warn",
        "reqwest=info",
        "rustls=info",
        "want=info",
        "tokio_util::codec=error",
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

    let registry = {
        let (non_blocking, stdio_guard) =
            tracing_appender::non_blocking(terminal.make_writer());
        guards.push(stdio_guard);
        registry.with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
    };

    registry.with(filter).init();

    if tracing::level_enabled!(tracing::Level::TRACE) {
        info!("Tracing Level: TRACE");
    } else if tracing::level_enabled!(tracing::Level::DEBUG) {
        info!("Tracing Level: DEBUG");
    };

    // Return the guards
    guards
}

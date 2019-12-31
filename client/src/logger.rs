use crate::Result;
use fern::colors::{Color, ColoredLevelConfig};

/// Setup logging.
pub fn log(level: log::LevelFilter) -> Result<()> {
    // TODO: instead of taking the log path from the config, find one in this file?
    /*
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::Green)
        .trace(Color::BrightBlack);

    let base = fern::Dispatch::new()
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("tokio_reactor", log::LevelFilter::Warn)
        .level_for("gfx_backend_vulkan", log::LevelFilter::Warn)
        .level_for("mio", log::LevelFilter::Debug)
        .level_for("want", log::LevelFilter::Debug);

    let file_cfg = fern::Dispatch::new()
        .level(log::LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}:{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record
                    .line()
                    .map(|x| x.to_string())
                    .unwrap_or("X".to_string()),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(&config.log_file)?);

    let stdout_cfg = fern::Dispatch::new()
        .level(level)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout());

    base.chain(file_cfg).chain(stdout_cfg).apply()?;
    */
    Ok(())
}

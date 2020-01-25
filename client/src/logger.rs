use crate::{Result, filesystem};
use fern::colors::{Color, ColoredLevelConfig};

/// Setup logging.
pub fn log(level: log::LevelFilter) -> Result<()> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::Green)
        .trace(Color::BrightBlack);

    let base = fern::Dispatch::new()
        .level_for("html5ever", log::LevelFilter::Error)
        .level_for("winit", log::LevelFilter::Warn)
        .level_for("wgpu_native", log::LevelFilter::Info)
        .level_for("strip_markdown", log::LevelFilter::Warn)
        .level_for("tokio_reactor", log::LevelFilter::Warn)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("iced_wgpu::renderer", log::LevelFilter::Info)
        .level_for("iced_winit", log::LevelFilter::Info)
        .level_for("wgpu_native", log::LevelFilter::Warn)
        .level_for("gfx_backend_vulkan", log::LevelFilter::Info);
    
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
        .chain(fern::log_file(&filesystem::get_log_path())?);
    
    let mut stdout_cfg = fern::Dispatch::new().level(level);
    // If more verbose debugging is requested. We will print the lines too.
    if level == log::LevelFilter::Trace {
        stdout_cfg = stdout_cfg.format(move |out, message, record| {
            out.finish(format_args!(
                "[{}:{}][{}] {}",
                record.target(),
                record
                    .line()
                    .map(|x| x.to_string())
                    .unwrap_or("X".to_string()),
                colors.color(record.level()),
                message
            ))
        });
    } else {
        stdout_cfg = stdout_cfg.format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        });
    }
    
    stdout_cfg = stdout_cfg.chain(std::io::stdout());

    base.chain(file_cfg)
        .chain(stdout_cfg)
        .apply()?;

    Ok(())
}

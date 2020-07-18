use crate::fs;
use fern::colors::{Color, ColoredLevelConfig};

/// Setup logging.
pub fn log(level: log::LevelFilter) {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::Green)
        .trace(Color::BrightBlack);

    let base = fern::Dispatch::new()
        .level_for("html5ever", log::LevelFilter::Error)
        .level_for("winit", log::LevelFilter::Error)
        .level_for("wgpu_native", log::LevelFilter::Info)
        .level_for("strip_markdown", log::LevelFilter::Warn)
        .level_for("tokio_reactor", log::LevelFilter::Warn)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("iced_wgpu::renderer", log::LevelFilter::Info)
        .level_for("iced_winit", log::LevelFilter::Info)
        .level_for("wgpu_native", log::LevelFilter::Warn)
        .level_for("gfx_backend_vulkan", log::LevelFilter::Info)
        .level_for("gfx_backend_dx12", log::LevelFilter::Info)
        .level_for("isahc", log::LevelFilter::Info)
        .level_for("iced_wgpu::image::atlas", log::LevelFilter::Warn)
        .level_for("wgpu_core", log::LevelFilter::Warn)
        .level_for("iced_wgpu::backend", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Info);

    let file_cfg = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .level_for("airshipper", log::LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}:{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record
                    .line()
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "X".to_string()),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(&fs::get_log_path()).expect("Failed to setup log file!"));

    let mut stdout_cfg = fern::Dispatch::new().level(level);
    // If more verbose debugging is requested. We will print the lines too.
    if level == log::LevelFilter::Debug || level == log::LevelFilter::Trace {
        stdout_cfg = stdout_cfg.format(move |out, message, record| {
            out.finish(format_args!(
                "[{}:{}][{}] {}",
                record.target(),
                record
                    .line()
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "X".to_string()),
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
        .apply()
        .expect("Failed to setup logging.");
}

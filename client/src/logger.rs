use crate::fs;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

const MAX_LOG_LINES: usize = 10_000;

/// Setup logging.
pub fn log(level: LevelFilter) {
    // Clean up log file if possible
    if fs::log_file().exists() {
        if let Ok(count) =
            std::fs::read_to_string(fs::log_file()).map(|x| x.lines().count())
        {
            if count > MAX_LOG_LINES {
                let _ = std::fs::remove_file(fs::log_file());
            }
        }
    }

    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::Green)
        .trace(Color::BrightBlack);

    let base = fern::Dispatch::new()
        .level_for("html5ever", LevelFilter::Error)
        .level_for("winit", LevelFilter::Error)
        .level_for("wgpu_native", LevelFilter::Info)
        .level_for("strip_markdown", LevelFilter::Warn)
        .level_for("tokio_reactor", LevelFilter::Warn)
        .level_for("hyper", LevelFilter::Warn)
        .level_for("iced_wgpu::renderer", LevelFilter::Info)
        .level_for("iced_winit", LevelFilter::Info)
        .level_for("wgpu_native", LevelFilter::Warn)
        .level_for("gfx_backend_vulkan", LevelFilter::Warn)
        .level_for("gfx_backend_dx12", LevelFilter::Info)
        .level_for("isahc", LevelFilter::Info)
        .level_for("iced_wgpu::image::atlas", LevelFilter::Warn)
        .level_for("wgpu_core", LevelFilter::Warn)
        .level_for("wgpu", LevelFilter::Warn)
        .level_for("iced_wgpu::backend", LevelFilter::Warn)
        .level_for("reqwest", LevelFilter::Info)
        .level_for("tracing", LevelFilter::Off);

    let file_cfg = fern::Dispatch::new()
        .level(LevelFilter::Info)
        .level_for("airshipper", LevelFilter::Debug)
        .level_for("output::Veloren", LevelFilter::Off)
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
        .chain(fern::log_file(&fs::log_file()).expect("Failed to setup log file!"));

    let mut stdout_cfg = fern::Dispatch::new()
        .level(level)
        .level_for("wgpu_core::device", LevelFilter::Error);
    // If more verbose debugging is requested. We will print the lines too.
    if level == LevelFilter::Debug || level == LevelFilter::Trace {
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

use chrono::Local;
use colored::*;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter;
use std::env;

pub fn setup_logger() -> Result<(), fern::InitError> {
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let level_filter = match log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red)
        .debug(Color::Blue)
        .trace(Color::Cyan);

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {} {}",
                Local::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
                    .dimmed(),
                colors.color(record.level()),
                message
            ))
        })
        .level(level_filter)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

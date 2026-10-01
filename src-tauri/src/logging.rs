use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};
use std::{backtrace::Backtrace, fs::OpenOptions, panic, path::Path};

use crate::errors::Result;

const LOG_FILE_NAME: &str = "todai.log";

/// Logs to the terminal and to `log_dir/todai.log`, and logs panics.
///
/// The log file is appended to, so that the logs of a crashed run survive the next launch.
pub fn init(log_dir: &Path) -> Result<()> {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let mut config = ConfigBuilder::new();
    // With the date, since the log file spans several runs.
    config.set_time_format_rfc3339();
    // Fails on some platforms when other threads are running: keep UTC timestamps then.
    let _ = config.set_time_offset_to_local();
    let config = config.build();

    std::fs::create_dir_all(log_dir)?;
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join(LOG_FILE_NAME))?;

    CombinedLogger::init(vec![
        TermLogger::new(level, config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(level, config, log_file),
    ])?;

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        log::error!("{info}\n{}", Backtrace::force_capture());
        default_hook(info);
    }));

    log::info!("Logging to {}", log_dir.join(LOG_FILE_NAME).display());
    Ok(())
}

use chrono::Local;
use eyre::Result;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use teamy_windows::log::LOG_BUFFER;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

use crate::home::PiingDirs;

pub fn initialize(
    level: Level,
    dirs: &PiingDirs,
    requested_log: Option<PathBuf>,
) -> Result<PathBuf> {
    let log_path = requested_log.unwrap_or_else(|| default_log_path(dirs));
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let log_file = File::create(&log_path)?;
    let shared_file = Arc::new(Mutex::new(log_file));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_writer(std::io::stderr.and(LOG_BUFFER.clone()))
        .pretty()
        .without_time();

    let json_writer = {
        let file = Arc::clone(&shared_file);
        BoxMakeWriter::new(move || {
            let guard = file.lock().expect("log file poisoned");
            guard
                .try_clone()
                .expect("failed to clone log file handle for json writer")
        })
    };

    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .with_writer(json_writer);

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::from_level(level).into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(json_layer)
        .try_init()
        .map_err(|e| eyre::eyre!(e))?;

    Ok(log_path)
}

pub fn default_log_path(dirs: &PiingDirs) -> PathBuf {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    dirs.logs_dir()
        .join(format!("piing_{}.log.ndjson", timestamp))
}

pub fn describe_log_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

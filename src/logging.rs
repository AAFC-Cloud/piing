use crate::home::PiingDirs;
use chrono::Local;
use eyre::Result;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use teamy_windows::log::LOG_BUFFER;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Clone)]
pub enum LogWritingBehaviour {
    TerminalOnly,
    TerminalAndDefaultFile,
    TerminalAndSpecificFile(PathBuf),
}

impl LogWritingBehaviour {
    pub fn log_output_path(self, dirs: &PiingDirs) -> Option<PathBuf> {
        match self {
            LogWritingBehaviour::TerminalOnly => None,
            LogWritingBehaviour::TerminalAndDefaultFile => Some({
                let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
                dirs.logs_dir()
                    .join(format!("piing_{}.log.ndjson", timestamp))
            }),
            LogWritingBehaviour::TerminalAndSpecificFile(path) => Some(path),
        }
    }
}

pub fn initialize(
    level: Level,
    dirs: &PiingDirs,
    behaviour: LogWritingBehaviour,
) -> Result<Option<PathBuf>> {
    let log_path = behaviour.log_output_path(dirs);

    let shared_file = if let Some(ref path) = log_path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let log_file = File::create(path)?;
        Some(Arc::new(Mutex::new(log_file)))
    } else {
        None
    };

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_writer(std::io::stderr.and(LOG_BUFFER.clone()))
        .pretty()
        .without_time();

    let json_layer = shared_file.map(|file| {
        let json_writer = BoxMakeWriter::new(move || {
            let guard = file.lock().expect("log file poisoned");
            guard
                .try_clone()
                .expect("failed to clone log file handle for json writer")
        });

        tracing_subscriber::fmt::layer()
            .json()
            .with_file(false)
            .with_line_number(false)
            .with_target(false)
            .with_writer(json_writer)
    });

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::from_level(level).into())
        .from_env_lossy();

    if let Some(json_layer) = json_layer {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stderr_layer)
            .with(json_layer)
            .try_init()
            .map_err(|e| eyre::eyre!(e))?;
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stderr_layer)
            .try_init()
            .map_err(|e| eyre::eyre!(e))?;
    }

    Ok(log_path)
}

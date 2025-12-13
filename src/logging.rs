use crate::config::ConfigSnapshot;
use crate::config::LatencyColouration;
use crate::home::PiingDirs;
use chrono::Local;
use eyre::Result;
use owo_colors::OwoColorize;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use teamy_windows::log::LOG_BUFFER;
use tracing::Level;
use tracing::Subscriber;
use tracing::debug;
use tracing::field::Field;
use tracing::field::Visit;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Clone)]
pub enum LogWritingBehaviour {
    TerminalOnly,
    TerminalAndDefaultFile,
    TerminalAndSpecificFile(PathBuf),
}

impl LogWritingBehaviour {
    #[must_use]
    pub fn log_output_path(self, dirs: &PiingDirs) -> Option<PathBuf> {
        match self {
            LogWritingBehaviour::TerminalOnly => None,
            LogWritingBehaviour::TerminalAndDefaultFile => Some({
                let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
                dirs.logs_dir()
                    .join(format!("piing_{timestamp}.log.ndjson"))
            }),
            LogWritingBehaviour::TerminalAndSpecificFile(path) => Some(path),
        }
    }
}

fn load_latency_colouration(dirs: &PiingDirs) -> LatencyColouration {
    match ConfigSnapshot::try_from_dir(dirs.config_dir()) {
        Ok(snapshot) => snapshot.latency_colouration().clone(),
        Err(error) => {
            // Logging is not yet initialised, so keep this minimal.
            eprintln!(
                "Failed to load latency colouration config: {error}. Falling back to defaults"
            );
            LatencyColouration::default()
        }
    }
}

/// # Errors
/// Returns an error if log file creation or initialization fails
///
/// # Panics
/// Panics if the log file mutex is poisoned
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

    let latency_colouration = load_latency_colouration(dirs);
    let show_location = matches!(level, Level::DEBUG | Level::TRACE);

    let stderr_layer = tracing_subscriber::fmt::layer()
        .event_format(
            PrettyLatencyEventFormat::new(latency_colouration)
                .with_target(false)
                .with_location(show_location),
        )
        .with_writer(std::io::stderr.and(LOG_BUFFER.clone()));

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

    debug!("Logging initialized");

    Ok(log_path)
}

#[derive(Debug, Clone)]
struct PrettyLatencyEventFormat {
    colouration: LatencyColouration,
    show_target: bool,
    show_location: bool,
}

impl PrettyLatencyEventFormat {
    fn new(colouration: LatencyColouration) -> Self {
        Self {
            colouration,
            show_target: true,
            show_location: true,
        }
    }

    fn with_target(mut self, show_target: bool) -> Self {
        self.show_target = show_target;
        self
    }

    fn with_location(mut self, show_location: bool) -> Self {
        self.show_location = show_location;
        self
    }

    fn format_latency_value(&self, value: &str, has_ansi: bool) -> String {
        if !has_ansi {
            return value.to_string();
        }
        let trimmed = value.trim_matches('"');
        if let Ok(latency_ms) = trimmed.parse::<f64>()
            && let Some(colour) = self.colouration.color_for(latency_ms)
        {
            return trimmed.color(colour).to_string();
        }
        value.to_string()
    }
}

impl<S, N> FormatEvent<S, N> for PrettyLatencyEventFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        let mut collector = FieldCollector::default();
        event.record(&mut collector);

        let metadata = event.metadata();
        let level = format_level(*metadata.level(), writer.has_ansi_escapes());
        writer.write_str(&level)?;

        if self.show_target && !metadata.target().is_empty() {
            write!(writer, " {}", metadata.target())?;
        }

        let mut wrote_body = false;
        if let Some(message) = collector.message.as_deref() {
            write!(writer, ": {message}")?;
            wrote_body = true;
        }

        for (index, (key, value)) in collector.fields.iter().enumerate() {
            let delimiter = if wrote_body || index > 0 { ", " } else { ": " };
            let rendered = if key == "latency_ms" {
                self.format_latency_value(value, writer.has_ansi_escapes())
            } else {
                value.clone()
            };
            write!(writer, "{delimiter}{key}: {rendered}")?;
            wrote_body = true;
        }

        writer.write_char('\n')?;
        if self.show_location
            && let (Some(file), Some(line)) = (metadata.file(), metadata.line())
        {
            writeln!(writer, "    at {file}:{line}")?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct FieldCollector {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl FieldCollector {
    fn push(&mut self, field: &Field, rendered: String) {
        if field.name() == "message" {
            self.message = Some(rendered.trim_matches('"').to_string());
        } else {
            self.fields.push((field.name().to_string(), rendered));
        }
    }
}

impl Visit for FieldCollector {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.push(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.push(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.push(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.push(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.push(field, value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.push(field, value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.push(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.push(field, format!("{value}"));
    }
}

fn format_level(level: Level, has_ansi: bool) -> String {
    let text = level.to_string();
    if !has_ansi {
        return text;
    }

    match level {
        Level::ERROR => text.red().to_string(),
        Level::WARN => text.yellow().to_string(),
        Level::INFO => text.green().to_string(),
        Level::DEBUG => text.blue().to_string(),
        Level::TRACE => text.magenta().to_string(),
    }
}

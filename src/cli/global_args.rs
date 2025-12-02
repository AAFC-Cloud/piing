use clap::Args;
use std::path::PathBuf;
use tracing::Level;

#[derive(Debug, Clone, Args)]
pub struct GlobalArgs {
    /// Enable verbose debug logging
    #[arg(long, global = true)]
    pub debug: bool,

    /// Write structured ndjson logs to this file instead of the default in $PIING_HOME/logs
    #[arg(long, global = true, value_name = "FILE")]
    pub log_file: Option<PathBuf>,
}

impl GlobalArgs {
    pub fn log_level(&self) -> Level {
        if self.debug {
            Level::DEBUG
        } else {
            Level::INFO
        }
    }
}

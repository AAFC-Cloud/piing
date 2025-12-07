use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct AuditArgs {}

impl AuditArgs {
    /// # Errors
    /// Returns an error if reading the logs directory fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        println!("Discovering log files...\n");

        let logs_dir = dirs.logs_dir();

        if !logs_dir.exists() {
            println!("Logs directory does not exist: {}", logs_dir.display());
            return Ok(());
        }

        let mut log_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(logs_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata()
                    && metadata.is_file()
                    && let Some(ext) = entry.path().extension()
                    && (ext == "ndjson" || ext == "log")
                {
                    log_files.push(entry.path());
                }
            }
        }

        if log_files.is_empty() {
            println!("No log files found in: {}", logs_dir.display());
        } else {
            log_files.sort();
            println!("Found {} log file(s):\n", log_files.len());
            for log_file in log_files {
                println!("  {}", log_file.display());
            }
        }

        Ok(())
    }
}

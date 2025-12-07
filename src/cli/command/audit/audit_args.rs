use crate::home::PiingDirs;
use chrono::DateTime;
use chrono::Datelike;
use chrono::Duration;
use chrono::Local;
use chrono::Timelike;
use clap::Args;
use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    fields: LogFields,
}

#[derive(Debug, Deserialize)]
struct LogFields {
    success: bool,
}

#[derive(Debug)]
struct PingEvent {
    timestamp: DateTime<Local>,
    success: bool,
}

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
            return Ok(());
        }

        log_files.sort();
        println!("Found {} log file(s)\n", log_files.len());

        // Parse all log entries
        let mut events: Vec<PingEvent> = Vec::new();

        for log_file in &log_files {
            let file = File::open(log_file)
                .wrap_err_with(|| format!("Failed to open log file: {}", log_file.display()))?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                    // Parse timestamp from ISO 8601 format
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                        events.push(PingEvent {
                            timestamp: dt.with_timezone(&Local),
                            success: entry.fields.success,
                        });
                    }
                }
            }
        }

        println!("Parsed {} ping events\n", events.len());

        // Count failures by hour of day (0-23)
        let mut failures_by_hour: HashMap<u32, usize> = HashMap::new();
        let mut total_by_hour: HashMap<u32, usize> = HashMap::new();

        for event in &events {
            let hour = event.timestamp.hour();
            *total_by_hour.entry(hour).or_insert(0) += 1;
            if !event.success {
                *failures_by_hour.entry(hour).or_insert(0) += 1;
            }
        }

        // Print histogram
        println!("Failure Distribution by Hour of Day:\n");
        println!("Hour | Failures | Total | Failure Rate | Bar");
        println!("-----|----------|-------|--------------|{}", "-".repeat(50));

        let max_failures = failures_by_hour.values().max().copied().unwrap_or(0);
        let bar_width = 50;

        for hour in 0..24 {
            let failures = failures_by_hour.get(&hour).copied().unwrap_or(0);
            let total = total_by_hour.get(&hour).copied().unwrap_or(0);
            #[allow(clippy::cast_precision_loss)]
            let rate = if total > 0 {
                (failures as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let bar_length = if max_failures > 0 {
                (failures * bar_width) / max_failures
            } else {
                0
            };

            let bar = "█".repeat(bar_length);

            println!("{hour:4} | {failures:8} | {total:5} | {rate:11.2}% | {bar}");
        }

        println!(
            "\nTotal failures: {}",
            failures_by_hour.values().sum::<usize>()
        );
        println!("Total events: {}", events.len());

        // Count failures by day of week (0=Monday, 6=Sunday)
        let mut failures_by_day: HashMap<u32, usize> = HashMap::new();
        let mut total_by_day: HashMap<u32, usize> = HashMap::new();

        for event in &events {
            let day = event.timestamp.weekday().num_days_from_monday();
            *total_by_day.entry(day).or_insert(0) += 1;
            if !event.success {
                *failures_by_day.entry(day).or_insert(0) += 1;
            }
        }

        // Print day of week histogram
        println!("\n\nFailure Distribution by Day of Week:\n");
        println!("Day       | Failures | Total | Failure Rate | Bar");
        println!("----------|----------|-------|--------------|{}", "-".repeat(50));

        let max_failures_day = failures_by_day.values().max().copied().unwrap_or(0);
        let day_names = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

        for day in 0..7 {
            let failures = failures_by_day.get(&day).copied().unwrap_or(0);
            let total = total_by_day.get(&day).copied().unwrap_or(0);
            #[allow(clippy::cast_precision_loss)]
            let rate = if total > 0 {
                (failures as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let bar_length = if max_failures_day > 0 {
                (failures * bar_width) / max_failures_day
            } else {
                0
            };

            let bar = "█".repeat(bar_length);

            println!(
                "{:9} | {:8} | {:5} | {:11.2}% | {}",
                day_names[day as usize], failures, total, rate, bar
            );
        }

        println!(
            "\nTotal failures: {}",
            failures_by_day.values().sum::<usize>()
        );
        println!("Total events: {}", events.len());

        // Filter events from last 24 hours
        let now = Local::now();
        let twenty_four_hours_ago = now - Duration::hours(24);
        let recent_events: Vec<&PingEvent> = events
            .iter()
            .filter(|e| e.timestamp >= twenty_four_hours_ago)
            .collect();

        if !recent_events.is_empty() {
            // Count failures by hour for last 24 hours
            let mut recent_failures_by_hour: HashMap<u32, usize> = HashMap::new();
            let mut recent_total_by_hour: HashMap<u32, usize> = HashMap::new();

            for event in &recent_events {
                let hour = event.timestamp.hour();
                *recent_total_by_hour.entry(hour).or_insert(0) += 1;
                if !event.success {
                    *recent_failures_by_hour.entry(hour).or_insert(0) += 1;
                }
            }

            // Print last 24 hours histogram
            println!("\n\nFailure Distribution by Hour of Day (Last 24 Hours):\n");
            println!("Hour | Failures | Total | Failure Rate | Bar");
            println!("-----|----------|-------|--------------|{}", "-".repeat(50));

            let max_recent_failures = recent_failures_by_hour.values().max().copied().unwrap_or(0);

            for hour in 0..24 {
                let failures = recent_failures_by_hour.get(&hour).copied().unwrap_or(0);
                let total = recent_total_by_hour.get(&hour).copied().unwrap_or(0);
                #[allow(clippy::cast_precision_loss)]
                let rate = if total > 0 {
                    (failures as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                let bar_length = if max_recent_failures > 0 {
                    (failures * bar_width) / max_recent_failures
                } else {
                    0
                };

                let bar = "█".repeat(bar_length);

                println!("{hour:4} | {failures:8} | {total:5} | {rate:11.2}% | {bar}");
            }

            println!(
                "\nTotal failures (last 24h): {}",
                recent_failures_by_hour.values().sum::<usize>()
            );
            println!("Total events (last 24h): {}", recent_events.len());
        } else {
            println!("\n\nNo events found in the last 24 hours.");
        }

        Ok(())
    }
}

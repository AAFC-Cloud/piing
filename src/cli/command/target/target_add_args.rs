use crate::config::ConfigPaths;
use crate::config::targets::build_block;
use crate::config::targets::sanitize_label;
use crate::ping::PingMode;
use crate::ping::parse_destination;
use clap::Args;
use eyre::Result;
use hcl::edit::structure::Body;
use std::time::Duration;

#[derive(Debug, Args)]
pub struct TargetAddArgs {
    /// Destination to ping (domain, IP, or URL)
    pub value: String,
    /// Optional target name; defaults to a sanitized version of the value
    #[arg(long)]
    pub name: Option<String>,
    /// Ping mode to use for this target
    #[arg(long, default_value = "icmp")]
    pub mode: PingMode,
    /// Interval between pings expressed with humantime syntax, e.g. "1s" or "2m"
    #[arg(long, default_value = "1s", value_parser = humantime::parse_duration)]
    pub interval: Duration,
}

impl TargetAddArgs {
    /// # Errors
    /// Returns an error if config operations fail
    pub fn invoke(self) -> Result<()> {
        if self.value.trim().is_empty() {
            eyre::bail!("Target value cannot be empty");
        }

        let paths = ConfigPaths::new();
        paths.ensure_defaults()?;

        let snapshot = paths.load_snapshot()?;
        let requested_name = self
            .name
            .as_deref()
            .map_or_else(|| sanitize_label(self.value.trim()), str::to_string);
        let sanitized = sanitize_label(&requested_name);
        if sanitized.is_empty() {
            eyre::bail!("Unable to derive a valid target name");
        }

        if snapshot
            .targets
            .iter()
            .any(|target| target.id.name.eq_ignore_ascii_case(&sanitized))
        {
            eyre::bail!("Target with name '{}' already exists", sanitized);
        }

        let destination = parse_destination(self.value.trim(), self.mode);
        let block = build_block(&sanitized, &destination, self.mode, self.interval);
        let body = Body::builder().block(block).build();
        let file_path = paths.unique_file_path(&sanitized);
        paths.write_body(&file_path, &body)?;

        println!(
            "Added target '{}' ({}) with {} mode every {} (file: {})",
            sanitized,
            self.value.trim(),
            self.mode.as_str(),
            humantime::format_duration(self.interval),
            file_path.display()
        );

        Ok(())
    }
}

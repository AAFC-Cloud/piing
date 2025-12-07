use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;
use std::time::Duration;

#[derive(Debug, Args)]
pub struct IntervalSetArgs {
    #[clap(value_parser = humantime::parse_duration)]
    pub interval: Duration,
}

impl IntervalSetArgs {
    /// # Errors
    /// Returns an error if writing the interval config fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        paths.write_interval(self.interval)?;
        println!(
            "Interval set to {}",
            humantime::format_duration(self.interval)
        );
        Ok(())
    }
}

use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct IntervalShowArgs {}

impl IntervalShowArgs {
    /// # Errors
    /// Returns an error if loading the config fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let snapshot = paths.load_snapshot()?;
        println!(
            "Current interval: {}",
            humantime::format_duration(snapshot.interval)
        );
        Ok(())
    }
}

use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct TargetListArgs {}

impl TargetListArgs {
    /// # Errors
    /// Returns an error if config operations fail
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let snapshot = paths.load_snapshot()?;
        let targets = &snapshot.targets;

        if targets.is_empty() {
            println!("No targets configured.");
        } else {
            for target in targets {
                println!(
                    "{:<20} {:<20} mode={} interval={}",
                    target.id.name,
                    target.value.display,
                    target.mode.as_str(),
                    humantime::format_duration(target.interval)
                );
            }
        }

        Ok(())
    }
}

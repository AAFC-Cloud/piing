use crate::config::Config;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct TargetListArgs {}

impl TargetListArgs {
    /// # Errors
    /// Returns an error if config operations fail
    pub fn invoke(self) -> Result<()> {
        let targets = &Config::current()?.targets;

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

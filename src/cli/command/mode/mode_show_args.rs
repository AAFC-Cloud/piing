use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct ModeShowArgs {}

impl ModeShowArgs {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let snapshot = paths.load_snapshot()?;
        println!("Current mode: {}", snapshot.mode.as_str());
        Ok(())
    }
}

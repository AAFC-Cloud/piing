use crate::config::ConfigPaths;
use crate::home::PiingDirs;
use crate::ping::PingMode;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct ModeSetArgs {
    pub mode: PingMode,
}

impl ModeSetArgs {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        paths.write_mode(self.mode)?;
        println!("Mode set to {}", self.mode.as_str());
        Ok(())
    }
}

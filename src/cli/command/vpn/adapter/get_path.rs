use clap::Args;
use eyre::Result;
use crate::home::PiingDirs;

#[derive(Debug, Default, Args)]
pub struct GetPathArgs {}

impl GetPathArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        println!("{}", dirs.vpn_adapter_criteria_dir().display());
        Ok(())
    }
}
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct HomeArgs {}

impl HomeArgs {
    /// # Errors
    /// Returns an error if retrieving the home directory fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        println!("{}", dirs.home_dir().display());
        Ok(())
    }
}

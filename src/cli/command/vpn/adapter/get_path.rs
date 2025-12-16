use crate::home::PIING_HOME;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct GetPathArgs {}

impl GetPathArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self) -> Result<()> {
        println!("{}", PIING_HOME.config_dir().display());
        Ok(())
    }
}

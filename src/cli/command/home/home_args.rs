use crate::home::PIING_HOME;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct HomeArgs {}

impl HomeArgs {
    /// # Errors
    /// Returns an error if retrieving the home directory fails
    pub fn invoke(self) -> Result<()> {
        println!("{}", PIING_HOME.display());
        Ok(())
    }
}

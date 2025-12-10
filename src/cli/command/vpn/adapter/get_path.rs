use crate::config::ConfigPaths;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct GetPathArgs {}

impl GetPathArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self, paths: &ConfigPaths) -> Result<()> {
        println!("{}", paths.config_dir().display());
        Ok(())
    }
}

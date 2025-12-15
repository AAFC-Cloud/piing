use crate::runtime;
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct RunArgs {}

impl RunArgs {
    /// # Errors
    /// Returns an error if the runtime fails to start
    pub fn invoke(self) -> Result<()> {
        runtime::run()
    }
}

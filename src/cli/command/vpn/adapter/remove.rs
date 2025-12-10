use clap::Args;
use eyre::Result;
use crate::home::PiingDirs;

#[derive(Debug, Default, Args)]
pub struct RemoveArgs {}

impl RemoveArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self, _dirs: &PiingDirs) -> Result<()> {
        // TODO: Implement removing VPN adapters
        println!("Stub: Remove VPN adapters");
        Ok(())
    }
}
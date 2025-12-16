// No longer require `ConfigPaths` for remove stub
use clap::Args;
use eyre::Result;

#[derive(Debug, Default, Args)]
pub struct RemoveArgs {}

impl RemoveArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self) -> Result<()> {
        // TODO: Implement removing VPN adapters
        println!("Stub: Remove VPN adapters");
        Ok(())
    }
}

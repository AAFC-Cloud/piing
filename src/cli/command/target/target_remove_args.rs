use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct TargetRemoveArgs {
    pub name: String,
}

impl TargetRemoveArgs {
    /// # Errors
    /// Returns an error if command execution fails
    pub fn invoke(self) -> Result<()> {
        println!(
            "Stub: target removal for '{}' is not implemented yet.",
            self.name
        );
        Ok(())
    }
}

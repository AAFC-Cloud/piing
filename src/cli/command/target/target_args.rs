use crate::cli::command::target::target_command::TargetCommand;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct TargetArgs {
    #[command(subcommand)]
    pub command: TargetCommand,
}

impl TargetArgs {
    /// # Errors
    /// Returns an error if the target command fails
    pub fn invoke(self) -> Result<()> {
        self.command.invoke()
    }
}

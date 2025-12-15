use crate::cli::command::sound::sound_command::SoundCommand;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct SoundArgs {
    #[command(subcommand)]
    pub command: SoundCommand,
}

impl SoundArgs {
    /// # Errors
    /// Returns an error if the sound command fails
    pub fn invoke(self) -> Result<()> {
        self.command.invoke()
    }
}

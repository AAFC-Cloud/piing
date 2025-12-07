use crate::cli::command::mode::mode_command::ModeCommand;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct ModeArgs {
    #[command(subcommand)]
    pub command: ModeCommand,
}

impl ModeArgs {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        self.command.invoke(dirs)
    }
}

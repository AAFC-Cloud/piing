use crate::cli::command::interval::interval_command::IntervalCommand;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct IntervalArgs {
    #[command(subcommand)]
    pub command: IntervalCommand,
}

impl IntervalArgs {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        self.command.invoke(dirs)
    }
}

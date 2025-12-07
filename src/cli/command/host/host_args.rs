use crate::cli::command::host::host_command::HostCommand;
use crate::home::PiingDirs;
use clap::Args;
use eyre::Result;

#[derive(Debug, Args)]
pub struct HostArgs {
    #[command(subcommand)]
    pub command: HostCommand,
}

impl HostArgs {
    /// # Errors
    /// Returns an error if the host command fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        self.command.invoke(dirs)
    }
}

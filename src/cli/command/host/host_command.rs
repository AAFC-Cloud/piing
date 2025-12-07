use crate::cli::command::host::host_add_args::HostAddArgs;
use crate::cli::command::host::host_list_args::HostListArgs;
use crate::cli::command::host::host_remove_args::HostRemoveArgs;
use crate::home::PiingDirs;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Subcommand)]
pub enum HostCommand {
    /// Add a host (domain, IP, or URL) to the monitored list
    Add(HostAddArgs),
    /// Remove a host from the monitored list
    Remove(HostRemoveArgs),
    /// List the configured hosts
    List(HostListArgs),
}

impl HostCommand {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        match self {
            HostCommand::Add(args) => args.invoke(dirs),
            HostCommand::Remove(args) => args.invoke(dirs),
            HostCommand::List(args) => args.invoke(dirs),
        }
    }
}

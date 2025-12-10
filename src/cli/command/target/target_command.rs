use crate::cli::command::target::target_add_args::TargetAddArgs;
use crate::cli::command::target::target_list_args::TargetListArgs;
use crate::cli::command::target::target_remove_args::TargetRemoveArgs;
use crate::home::PiingDirs;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Subcommand)]
pub enum TargetCommand {
    /// Add a target (domain, IP, or URL) to the monitored list
    Add(TargetAddArgs),
    /// Remove a target from the monitored list
    Remove(TargetRemoveArgs),
    /// List the configured targets
    List(TargetListArgs),
}

impl TargetCommand {
    /// # Errors
    /// Returns an error if the target subcommand fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        match self {
            TargetCommand::Add(args) => args.invoke(dirs),
            TargetCommand::Remove(args) => args.invoke(dirs),
            TargetCommand::List(args) => args.invoke(dirs),
        }
    }
}

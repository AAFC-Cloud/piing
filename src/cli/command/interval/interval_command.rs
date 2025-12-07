use crate::cli::command::interval::interval_set_args::IntervalSetArgs;
use crate::cli::command::interval::interval_show_args::IntervalShowArgs;
use crate::home::PiingDirs;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Subcommand)]
pub enum IntervalCommand {
    /// Set the ping interval
    Set(IntervalSetArgs),
    /// Show the current interval
    Show(IntervalShowArgs),
}

impl IntervalCommand {
    /// # Errors
    /// Returns an error if the interval subcommand fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        match self {
            IntervalCommand::Set(args) => args.invoke(dirs),
            IntervalCommand::Show(args) => args.invoke(dirs),
        }
    }
}

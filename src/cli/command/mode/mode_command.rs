use crate::cli::command::mode::mode_set_args::ModeSetArgs;
use crate::cli::command::mode::mode_show_args::ModeShowArgs;
use crate::home::PiingDirs;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Subcommand)]
pub enum ModeCommand {
    /// Set the ping mode
    Set(ModeSetArgs),
    /// Show the current ping mode
    Show(ModeShowArgs),
}

impl ModeCommand {
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        match self {
            ModeCommand::Set(args) => args.invoke(dirs),
            ModeCommand::Show(args) => args.invoke(dirs),
        }
    }
}

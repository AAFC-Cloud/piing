pub mod command;
pub mod global_args;

use clap::Parser;

use crate::cli::command::Command;
use crate::cli::global_args::GlobalArgs;
use crate::home::PiingDirs;

#[derive(Debug, Parser)]
#[command(
    name = "piing",
    version,
    about = "TeamDman's Windows tray ping utility"
)]
pub struct Cli {
    #[clap(flatten)]
    pub global_args: GlobalArgs,
    #[clap(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub fn invoke(self) -> eyre::Result<()> {
        let dirs = PiingDirs::ensure()?;
        let command = self.command.unwrap_or_default();
        command.invoke(self.global_args, dirs)
    }
}

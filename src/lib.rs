pub mod cli;
pub mod config;
pub mod home;
pub mod logging;
pub mod ping;
pub mod runtime;
pub mod tray;

use clap::CommandFactory;
use clap::FromArgMatches;

use crate::cli::Cli;

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::command();
    let cli = Cli::from_arg_matches(&cli.get_matches())?;
    cli.invoke()
}

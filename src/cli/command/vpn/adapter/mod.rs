pub mod add;
pub mod get_path;
pub mod list;
pub mod remove;

// `ConfigPaths` not required here; use global helpers where necessary
use clap::Args;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Args)]
pub struct AdapterArgs {
    #[command(subcommand)]
    pub command: AdapterCommand,
}

#[derive(Debug, Subcommand)]
pub enum AdapterCommand {
    /// Add VPN adapters
    Add(add::AddArgs),
    /// Remove VPN adapters
    Remove(remove::RemoveArgs),
    /// List VPN adapters
    List(list::ListArgs),
    /// Get path to adapter criteria directory
    GetPath(get_path::GetPathArgs),
}

impl AdapterArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self) -> Result<()> {
        match self.command {
            AdapterCommand::Add(args) => args.invoke()?,
            AdapterCommand::Remove(args) => args.invoke()?,
            AdapterCommand::List(args) => args.invoke()?,
            AdapterCommand::GetPath(args) => args.invoke()?,
        }
        Ok(())
    }
}

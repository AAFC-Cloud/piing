pub mod adapter;
pub mod check;

use crate::config::ConfigPaths;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

#[derive(Debug, Args)]
pub struct VpnArgs {
    #[command(subcommand)]
    pub command: VpnCommand,
}

#[derive(Debug, Subcommand)]
pub enum VpnCommand {
    /// Check if VPN is connected, exit 0 if a vpn connection is active, 1 if not
    Check(check::CheckArgs),
    /// Manage VPN adapters
    Adapter(adapter::AdapterArgs),
}

impl VpnArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self) -> Result<()> {
        let paths = ConfigPaths::new();
        paths.ensure_defaults()?;

        match self.command {
            VpnCommand::Check(args) => args.invoke(&paths)?,
            VpnCommand::Adapter(args) => args.invoke(&paths)?,
        }
        Ok(())
    }
}

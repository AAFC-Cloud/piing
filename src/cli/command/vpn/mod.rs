pub mod adapter;
pub mod check;

use crate::config::VpnCriteria;
use crate::home::PiingDirs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;
use tracing::debug;

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
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        match self.command {
            VpnCommand::Check(args) => {
                let criteria = VpnCriteria::try_from_dir(dirs.vpn_adapter_criteria_dir())?;
                debug!(count = criteria.0.len(), "Loaded VPN criteria");

                let exit_code = if args.invoke(&criteria)? {
                    0
                } else {
                    1
                };
                std::process::exit(exit_code);
            }
            VpnCommand::Adapter(args) => args.invoke(dirs)?,
        }
        Ok(())
    }
}

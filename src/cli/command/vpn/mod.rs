pub mod adapter;
pub mod check;

use crate::config::ConfigPaths;
use crate::config::vpn_criterion::VpnCriterion;
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
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;

        match self.command {
            VpnCommand::Check(args) => {
                let criteria = load_vpn_criteria(&paths)?;
                debug!(count = criteria.len(), "Loaded VPN criteria");

                let exit_code = i32::from(!args.invoke(&criteria)?);
                std::process::exit(exit_code);
            }
            VpnCommand::Adapter(args) => args.invoke(&paths)?,
        }
        Ok(())
    }
}

fn load_vpn_criteria(paths: &ConfigPaths) -> Result<Vec<VpnCriterion>> {
    let snapshot = paths.load_snapshot()?;
    Ok(snapshot.vpn_criteria().to_vec())
}

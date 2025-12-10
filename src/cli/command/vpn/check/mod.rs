use crate::config::vpn::VpnCriterion;
use clap::Args;
use eyre::Result;
use teamy_windows::network::NetworkAdapterExt;
use teamy_windows::network::NetworkAdapters;
use teamy_windows::network::OperStatusExt;
use tracing::debug;
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Suppress output and exit early on VPN found
    #[arg(short, long)]
    pub quiet: bool,
}

impl CheckArgs {
    /// # Errors
    /// Returns an error if the process check fails
    pub fn invoke(self, criteria: &[VpnCriterion]) -> Result<bool> {
        let adapters = NetworkAdapters::new()?;
        debug!(count = adapters.iter().count(), "Loaded network adapters");

        let mut active_vpn_found = false;
        for adapter in &adapters {
            let is_vpn = if criteria.iter().any(|c| c.matches(adapter)) {
                if adapter.peOperStatus == IfOperStatusUp {
                    active_vpn_found = true;
                    if self.quiet {
                        return Ok(true);
                    }
                }
                true
            } else {
                false
            };

            debug!(
                display_name = %adapter.display_name(),
                oper_status = %adapter.peOperStatus.display(),
                is_vpn,
            );
        }
        if !self.quiet {
            println!("{active_vpn_found}");
        }
        if !active_vpn_found {
            debug!("No active VPN adapters found.");
        }
        Ok(active_vpn_found)
    }
}

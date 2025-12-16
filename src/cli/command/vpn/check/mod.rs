use crate::config::Config;
use crate::vpn_detector::VpnDetector;
use clap::Args;
use eyre::Result;
use tracing::debug;
use tracing::warn;

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Suppress output and exit early on VPN found
    #[arg(short, long)]
    pub quiet: bool,
}

impl CheckArgs {
    /// # Errors
    /// Returns an error if the process check fails
    /// Invoke the command using config from `paths` (CLI-facing). This
    /// will print results and exit with an appropriate code for CLI use.
    pub fn invoke(self) -> Result<()> {
        let snapshot = Config::current()?;
        let vpn_criteria = &snapshot.vpn_criteria;
        let snapshot_time = snapshot.snapshot_time;

        let active = if vpn_criteria.is_empty() {
            warn!("No VPN criteria configured; skipping adapter checks");
            false
        } else {
            let mut detector = VpnDetector::new();
            detector.is_vpn_active(vpn_criteria, snapshot_time)
        };

        if !self.quiet {
            println!("{active}");
        }
        if !active {
            debug!("No active VPN adapters found.");
        }

        let exit_code = i32::from(!active);
        std::process::exit(exit_code);
    }
}

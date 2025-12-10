use clap::Args;
use eyre::Result;
use crate::config::VpnCriteria;
use crate::home::PiingDirs;
use owo_colors::{OwoColorize, Style};
use std::collections::HashSet;
use teamy_windows::network::{NetworkAdapterExt, NetworkAdapters};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Show all adapters, not just active ones
    #[arg(long)]
    all: bool,
}

impl ListArgs {
    /// # Errors
    /// Returns an error if the command fails
    pub fn invoke(self, dirs: &PiingDirs) -> Result<()> {
        let adapters = NetworkAdapters::new()?;
        let criteria = VpnCriteria::try_from_dir(dirs.vpn_adapter_criteria_dir()).unwrap_or(VpnCriteria(vec![]));
        let mut matched_criteria = HashSet::new();

        // Collect matched criteria from all adapters
        for adapter in adapters.iter() {
            let is_vpn = criteria.0.iter().any(|c| c.matches(adapter));
            if is_vpn {
                matched_criteria.insert(adapter.display_name().to_string());
            }
        }

        println!("Network Adapters:");
        for adapter in adapters.iter() {
            let is_up = adapter.peOperStatus == IfOperStatusUp;
            if !self.all && !is_up {
                continue;
            }
            let is_vpn = criteria.0.iter().any(|c| c.matches(adapter));
            let emoji = if !is_up { "⭕" } else if is_vpn { "🔒" } else { "🌐" };
            let name = adapter.display_name();
            let style = if is_vpn { Style::new().green() } else { Style::new().blue() };
            let colored_name = name.style(style);
            println!("  {} {}", emoji, colored_name);
        }

        println!("\nVPN Criteria:");
        for criterion in &criteria.0 {
            if !matched_criteria.contains(&criterion.block_label) {
                println!("  ❓ {} ({})", criterion.block_label, criterion.properties.display_name.as_deref().unwrap_or("").dimmed());
            } else {
                println!("  ✅ {} ({})", criterion.block_label, criterion.properties.display_name.as_deref().unwrap_or("").green());
            }
        }

        Ok(())
    }
}
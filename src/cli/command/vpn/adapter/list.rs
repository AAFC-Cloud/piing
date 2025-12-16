// uses `current_snapshot()` from `crate::config`
use crate::config::Config;
use clap::Args;
use eyre::Result;
use owo_colors::OwoColorize;
use owo_colors::Style;
use std::collections::HashSet;
use teamy_windows::network::NetworkAdapterExt;
use teamy_windows::network::NetworkAdapters;
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
    pub fn invoke(self) -> Result<()> {
        let adapters = NetworkAdapters::new()?;
        let criteria = &Config::current()?.vpn_criteria;
        let mut matched_criteria: HashSet<String> = HashSet::new();

        // Collect matched criteria from all adapters
        for adapter in &adapters {
            let is_vpn = criteria
                .iter()
                .any(|c: &crate::config::VpnCriterion| c.matches(adapter));
            if is_vpn {
                matched_criteria.insert(adapter.display_name().to_string());
            }
        }

        println!("Network Adapters:");
        for adapter in &adapters {
            let is_up = adapter.peOperStatus == IfOperStatusUp;
            if !self.all && !is_up {
                continue;
            }
            let is_vpn = criteria
                .iter()
                .any(|c: &crate::config::VpnCriterion| c.matches(adapter));
            let emoji = if !is_up {
                "⭕"
            } else if is_vpn {
                "🔒"
            } else {
                "🌐"
            };
            let name = adapter.display_name();
            let style = if is_vpn {
                Style::new().green()
            } else {
                Style::new().blue()
            };
            let colored_name = name.style(style);
            println!("  {emoji} {colored_name}");
        }

        println!("\nVPN Criteria:");
        for criterion in criteria {
            let name = criterion.display_name.as_deref().unwrap_or("(unnamed)");
            if matched_criteria.contains(name) {
                println!("  ✅ {}", name.green());
            } else {
                println!("  ❓ {}", name.dimmed());
            }
        }

        Ok(())
    }
}

use crate::config::vpn_criterion::VpnCriterion;
use eyre::Result;
use std::time::Duration;
use std::time::Instant;
use teamy_windows::network::NetworkAdapters;
use teamy_windows::network::NetworkInterfaceId;
use teamy_windows::network::NetworkInterfaceMonitor;
use tracing::debug;
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;

/// Background helper that maintains a cached list of adapters that match
/// the configured VPN criteria and checks their operational state via
/// `GetIfEntry2` (cheap) instead of re-enumerating adapters every tick.
pub(crate) struct VpnDetector {
    matched_ids: Vec<NetworkInterfaceId>,
    last_refresh: Option<Instant>,
    refresh_interval: Duration,
    last_criteria_names: Vec<Option<String>>,
}

impl VpnDetector {
    pub fn new() -> Self {
        Self {
            matched_ids: Vec::new(),
            last_refresh: None,
            refresh_interval: Duration::from_secs(30),
            last_criteria_names: Vec::new(),
        }
    }

    pub fn update_matches(&mut self, criteria: &[VpnCriterion]) -> Result<()> {
        let adapters = NetworkAdapters::new()?;
        debug!(
            count = adapters.iter().count(),
            "Refreshing network adapter list for VPN detection"
        );
        self.matched_ids = adapters
            .iter()
            .filter(|adapter| criteria.iter().any(|c| c.matches(adapter)))
            .map(NetworkInterfaceId::from)
            .collect();
        self.last_refresh = Some(Instant::now());
        self.last_criteria_names = criteria.iter().map(|c| c.display_name.clone()).collect();
        Ok(())
    }

    #[allow(clippy::collapsible_if)]
    pub fn is_vpn_active(&mut self, criteria: &[VpnCriterion], snapshot_time: Instant) -> bool {
        // Rebuild matches if criteria changed, our cache expired, or the
        // provided snapshot is newer than our last refresh.
        let criteria_names: Vec<Option<String>> =
            criteria.iter().map(|c| c.display_name.clone()).collect();
        let needs_refresh = match self.last_refresh {
            None => true,
            Some(t) => t.elapsed() > self.refresh_interval || snapshot_time > t,
        } || self.last_criteria_names != criteria_names;

        if needs_refresh {
            if let Err(e) = self.update_matches(criteria) {
                debug!(error = %e, "Failed to refresh adapter list; using cached entries if present");
            }
        }

        // Check per-interface operational status which is cheap compared to
        // a full GetAdaptersAddresses call.
        for id in &self.matched_ids {
            match NetworkInterfaceMonitor::new(*id) {
                Ok(monitor) => {
                    if monitor.oper_status() == IfOperStatusUp {
                        debug!(?id, "Found active VPN adapter");
                        return true;
                    }
                }
                Err(e) => {
                    // The interface may have gone away; force a refresh next time.
                    debug!(error = %e, ?id, "Failed to monitor interface; scheduling full refresh");
                    self.last_refresh = None;
                }
            }
        }
        debug!(
            matched = self.matched_ids.len(),
            "Found matching adapters for VPN criteria"
        );
        false
    }
}

use crate::config::VpnCriteria;
use crate::ping::PingMode;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub hosts: Vec<String>,
    pub mode: PingMode,
    pub interval: Duration,
    pub vpn_criteria: VpnCriteria,
}
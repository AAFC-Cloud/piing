use crate::config::ConfigSnapshot;
use crate::config::VpnCriteria;
use crate::home::PiingDirs;
use crate::ping::PingMode;
use eyre::Context;
use eyre::Result;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    hosts: PathBuf,
    mode: PathBuf,
    interval: PathBuf,
    vpn_criteria_dir: PathBuf,
}

impl ConfigPaths {
    #[must_use]
    pub fn new(dirs: &PiingDirs) -> Self {
        Self {
            hosts: dirs.hosts_file(),
            mode: dirs.mode_file(),
            interval: dirs.interval_file(),
            vpn_criteria_dir: dirs.vpn_adapter_criteria_dir().to_path_buf(),
        }
    }

    /// # Errors
    /// Returns an error if writing default files fails
    pub fn ensure_defaults(&self) -> Result<()> {
        if !self.hosts.exists() {
            fs::write(&self.hosts, "teksavvy.ca\n")
                .wrap_err("Failed to write default hosts file")?;
        }
        if !self.mode.exists() {
            fs::write(&self.mode, "icmp").wrap_err("Failed to write default mode file")?;
        }
        if !self.interval.exists() {
            fs::write(&self.interval, "1s").wrap_err("Failed to write default interval file")?;
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if writing the hosts file fails
    pub fn write_hosts(&self, hosts: &[String]) -> Result<()> {
        let mut data = String::new();
        for host in hosts {
            if host.trim().is_empty() {
                continue;
            }
            data.push_str(host.trim());
            data.push('\n');
        }
        fs::write(&self.hosts, data).wrap_err("Failed to write hosts file")
    }

    /// # Errors
    /// Returns an error if writing the mode file fails
    pub fn write_mode(&self, mode: PingMode) -> Result<()> {
        fs::write(&self.mode, mode.as_str()).wrap_err("Failed to write mode file")
    }

    /// # Errors
    /// Returns an error if writing the interval file fails
    pub fn write_interval(&self, interval: Duration) -> Result<()> {
        fs::write(
            &self.interval,
            humantime::format_duration(interval).to_string(),
        )
        .wrap_err("Failed to write interval file")
    }

    /// # Errors
    /// Returns an error if reading config files fails
    pub fn load_snapshot(&self) -> Result<ConfigSnapshot> {
        let hosts = if self.hosts.exists() {
            fs::read_to_string(&self.hosts)?
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect()
        } else {
            vec![]
        };

        let mode_str = if self.mode.exists() {
            fs::read_to_string(&self.mode)?
        } else {
            "icmp".to_string()
        };
        let mode = match mode_str.trim().to_lowercase().as_str() {
            "tcp" => PingMode::Tcp,
            "http-get" => PingMode::HttpGet,
            "http-head" => PingMode::HttpHead,
            _ => PingMode::Icmp,
        };

        let interval_str = if self.interval.exists() {
            fs::read_to_string(&self.interval)?
        } else {
            "1s".to_string()
        };
        let interval = humantime::parse_duration(interval_str.trim())
            .unwrap_or_else(|_| Duration::from_secs(1));

        let vpn_criteria =
            VpnCriteria::try_from_dir(&self.vpn_criteria_dir).unwrap_or(VpnCriteria(vec![]));

        Ok(ConfigSnapshot {
            hosts,
            mode,
            interval,
            vpn_criteria,
        })
    }

    #[must_use]
    pub fn hosts_path(&self) -> &PathBuf {
        &self.hosts
    }

    #[must_use]
    pub fn mode_path(&self) -> &PathBuf {
        &self.mode
    }

    #[must_use]
    pub fn interval_path(&self) -> &PathBuf {
        &self.interval
    }

    #[must_use]
    pub fn vpn_criteria_dir(&self) -> &PathBuf {
        &self.vpn_criteria_dir
    }
}

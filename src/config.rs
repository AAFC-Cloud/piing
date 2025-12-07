use crate::home::PiingDirs;
use crate::ping::PingMode;
use eyre::Context;
use eyre::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub hosts: Vec<String>,
    pub mode: PingMode,
    pub interval: Duration,
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    inner: Arc<RwLock<ConfigSnapshot>>,
}

impl ConfigStore {
    #[must_use]
    pub fn new(initial: ConfigSnapshot) -> Self {
        Self {
            inner: Arc::new(RwLock::new(initial)),
        }
    }

    /// # Panics
    /// Panics if the internal lock is poisoned
    #[must_use]
    pub fn snapshot(&self) -> ConfigSnapshot {
        self.inner.read().unwrap().clone()
    }

    /// # Panics
    /// Panics if the internal lock is poisoned
    pub fn replace(&self, snapshot: ConfigSnapshot) {
        *self.inner.write().unwrap() = snapshot;
    }
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    hosts: PathBuf,
    mode: PathBuf,
    interval: PathBuf,
}

impl ConfigPaths {
    #[must_use]
    pub fn new(dirs: &PiingDirs) -> Self {
        Self {
            hosts: dirs.hosts_file(),
            mode: dirs.mode_file(),
            interval: dirs.interval_file(),
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

        Ok(ConfigSnapshot {
            hosts,
            mode,
            interval,
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
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub paths: ConfigPaths,
    pub store: ConfigStore,
}

impl ConfigManager {
    /// # Errors
    /// Returns an error if config initialization fails
    pub fn initialize(dirs: &PiingDirs) -> Result<Self> {
        let paths = ConfigPaths::new(dirs);
        paths.ensure_defaults()?;
        let snapshot = paths.load_snapshot()?;
        let store = ConfigStore::new(snapshot);
        Ok(Self { paths, store })
    }

    /// # Errors
    /// Returns an error if reloading config files fails
    pub fn reload(&self) -> Result<ConfigSnapshot> {
        let snapshot = self.paths.load_snapshot()?;
        self.store.replace(snapshot.clone());
        Ok(snapshot)
    }
}

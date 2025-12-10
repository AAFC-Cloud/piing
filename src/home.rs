use eyre::Context;
use eyre::Result;
use std::env;
use std::ffi::c_void;
use std::path::Path;
use std::path::PathBuf;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::FOLDERID_RoamingAppData;
use windows::Win32::UI::Shell::KF_FLAG_DEFAULT;
use windows::Win32::UI::Shell::SHGetKnownFolderPath;

#[derive(Debug, Clone)]
pub struct PiingDirs {
    home: PathBuf,
    logs: PathBuf,
    config: PathBuf,
    vpn_config: PathBuf,
    vpn_adapter_criteria_dir: PathBuf,
}

impl PiingDirs {
    /// # Errors
    /// Returns an error if directory creation fails
    pub fn ensure() -> Result<Self> {
        let base = resolve_home_dir()?;
        std::fs::create_dir_all(&base).wrap_err("Failed to create PIING home directory")?;

        let home = base.canonicalize().unwrap_or(base.clone());
        let logs = home.join("logs");
        let config = home.join("config");
        let vpn_config = config.join("vpn");
        let vpn_adapter_criteria_dir = vpn_config.join("adapter_criteria");
        std::fs::create_dir_all(&logs).wrap_err("Failed to create logs directory")?;
        std::fs::create_dir_all(&config).wrap_err("Failed to create config directory")?;
        std::fs::create_dir_all(&vpn_config).wrap_err("Failed to create vpn config directory")?;
        std::fs::create_dir_all(&vpn_adapter_criteria_dir)
            .wrap_err("Failed to create vpn adapter criteria directory")?;

        Ok(Self {
            home,
            logs,
            config,
            vpn_config,
            vpn_adapter_criteria_dir,
        })
    }

    #[must_use]
    pub fn home_dir(&self) -> &Path {
        &self.home
    }

    #[must_use]
    pub fn logs_dir(&self) -> &Path {
        &self.logs
    }

    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config
    }

    #[must_use]
    pub fn vpn_config_dir(&self) -> &Path {
        &self.vpn_config
    }

    #[must_use]
    pub fn vpn_adapter_criteria_dir(&self) -> &Path {
        &self.vpn_adapter_criteria_dir
    }

    #[must_use]
    pub fn hosts_file(&self) -> PathBuf {
        self.config.join("hosts.txt")
    }

    #[must_use]
    pub fn mode_file(&self) -> PathBuf {
        self.config.join("mode.txt")
    }

    #[must_use]
    pub fn interval_file(&self) -> PathBuf {
        self.config.join("interval.txt")
    }
}

fn resolve_home_dir() -> Result<PathBuf> {
    if let Some(custom) = env::var_os("PIING_HOME") {
        let path = PathBuf::from(custom);
        if !path.exists() {
            std::fs::create_dir_all(&path).wrap_err("Failed to create PIING_HOME directory")?;
        }
        return Ok(path);
    }

    let raw_path = unsafe { SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, None) }
        .wrap_err("Failed to resolve %APPDATA%")?;
    let owned = unsafe { raw_path.to_string() }.wrap_err("Failed to convert path to string")?;
    unsafe { CoTaskMemFree(Some(raw_path.0 as *const c_void)) };
    Ok(PathBuf::from(owned).join("TeamDman").join("piing"))
}

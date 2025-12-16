use crate::config::ConfigSnapshot;
use crate::home::PIING_HOME;
use chrono::Utc;
use eyre::Context as _;
use eyre::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::RwLock;

/// Global holder for the current config snapshot (or the error encountered when loading it).
/// Use [`Config::current()`] to get the latest `ConfigSnapshot`.
/// Use [`Config::load()`] to refresh it from disk.
static CONFIG: LazyLock<RwLock<Result<ConfigSnapshot>>> = LazyLock::new(|| {
    let initial = ConfigSnapshot::try_from_dir(PIING_HOME.config_dir());
    RwLock::new(initial)
});

/// Generate a unique file path using the global config dir (under
/// `PIING_HOME.config_dir()`). Mirrors the previous behaviour on
/// `ConfigPaths::unique_file_path`.
/// Zero-sized convenience type for accessing global config helpers as
/// associated functions: `crate::config::Config::current()`, etc.
#[derive(Debug, Clone, Copy)]
pub struct Config;

impl Config {
    /// # Panics
    /// Panics if the global config lock is poisoned
    /// Return the current in-memory snapshot.
    ///
    /// # Errors
    /// Returns an error if the most recent attempt to load configuration
    /// files failed; the underlying error message will be returned.
    pub fn current() -> Result<ConfigSnapshot> {
        let guard = CONFIG.read().unwrap();
        match guard.as_ref() {
            Ok(snapshot) => Ok(snapshot.clone()),
            Err(e) => Err(eyre::eyre!(e.to_string())),
        }
    }

    /// Load the configuration from disk and replace the global snapshot.
    ///
    /// # Errors
    /// Returns an error if reading or parsing any config file fails.
    ///
    /// # Panics
    /// Panics if the internal `RwLock` for `CONFIG` is poisoned.
    pub fn load() -> Result<ConfigSnapshot> {
        let snapshot = ConfigSnapshot::try_from_dir(PIING_HOME.config_dir())?;
        *CONFIG.write().unwrap() = Ok(snapshot.clone());
        Ok(snapshot)
    }

    /// Generate a unique file path in the config directory for the given `stem`.
    #[must_use]
    pub fn unique_file_path(stem: &str) -> PathBuf {
        let timestamp = Utc::now().format("%Y-%m-%d_%H%M%S");
        let mut candidate = format!("{timestamp}_{stem}.piing_hcl");
        let mut counter = 1;
        loop {
            let path = PIING_HOME.config_dir().join(&candidate);
            if !path.exists() {
                return path;
            }
            counter += 1;
            candidate = format!("{timestamp}_{stem}_{counter}.piing_hcl");
        }
    }

    /// Write a HCL `Body` to `path` creating parent directories as required.
    ///
    /// # Errors
    /// Returns an error if directory creation or file write fails.
    pub fn write_body(path: &std::path::Path, body: &hcl::edit::structure::Body) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).wrap_err("Failed to create config parent directory")?;
        }
        fs::write(path, body.to_string())
            .wrap_err_with(|| format!("Failed to write config file: {}", path.display()))
    }
}

use crate::config::ConfigSnapshot;
use crate::home::PIING_HOME;
use chrono::Utc;
use eyre::Context;
use eyre::Result;
use hcl::edit::structure::Body;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    config_dir: PathBuf,
}
impl Default for ConfigPaths {
    fn default() -> Self {
        Self::new()
    }
}
impl ConfigPaths {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_dir: PIING_HOME.config_dir(),
        }
    }

    /// # Errors
    /// Returns an error if the config directory cannot be created
    pub fn ensure_defaults(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)
            .wrap_err("Failed to ensure piing config directory exists")
    }

    /// # Errors
    /// Returns an error if reading or parsing any config file fails
    pub fn load_snapshot(&self) -> Result<ConfigSnapshot> {
        ConfigSnapshot::try_from_dir(self.config_dir())
    }

    /// # Errors
    /// Returns an error if writing the body fails
    pub fn write_body(&self, path: &Path, body: &Body) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).wrap_err("Failed to create config parent directory")?;
        }
        fs::write(path, body.to_string())
            .wrap_err_with(|| format!("Failed to write config file: {}", path.display()))
    }

    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    #[must_use]
    pub fn unique_file_path(&self, stem: &str) -> PathBuf {
        let timestamp = Utc::now().format("%Y-%m-%d_%H%M%S");
        let mut candidate = format!("{timestamp}_{stem}.piing_hcl");
        let mut counter = 1;
        loop {
            let path = self.config_dir.join(&candidate);
            if !path.exists() {
                return path;
            }
            counter += 1;
            candidate = format!("{timestamp}_{stem}_{counter}.piing_hcl");
        }
    }
}

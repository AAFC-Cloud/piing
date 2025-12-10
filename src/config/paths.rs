use crate::config::ConfigSnapshot;
use crate::config::targets::build_block;
use crate::home::PiingDirs;
use crate::ping::PingMode;
use chrono::Utc;
use eyre::Context;
use eyre::Result;
use hcl::edit::parser::parse_body;
use hcl::edit::structure::Body;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    config_dir: PathBuf,
}

impl ConfigPaths {
    #[must_use]
    pub fn new(dirs: &PiingDirs) -> Self {
        Self {
            config_dir: dirs.config_dir().to_path_buf(),
        }
    }

    /// # Errors
    /// Returns an error if the config directory cannot be created
    pub fn ensure_defaults(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)
            .wrap_err("Failed to ensure piing config directory exists")?;
        if !self.has_hcl_files()? {
            let block = build_block(
                "default_target",
                "8.8.8.8",
                PingMode::Icmp,
                Duration::from_secs(1),
            );
            let body = Body::builder().block(block).build();
            let default_path = self.unique_file_path("default_target");
            self.write_body(&default_path, &body)?;
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if reading or parsing any config file fails
    pub fn load_snapshot(&self) -> Result<ConfigSnapshot> {
        let mut files = BTreeMap::new();
        if self.config_dir.exists() {
            for entry in fs::read_dir(&self.config_dir)? {
                let entry = entry?;
                let path = entry.path();
                if !is_hcl_file(&path) {
                    continue;
                }
                let content = fs::read_to_string(&path)
                    .wrap_err_with(|| format!("Failed to read config file: {}", path.display()))?;
                let body: Body = parse_body(&content)
                    .wrap_err_with(|| format!("Failed to parse config file: {}", path.display()))?;
                files.insert(path, body);
            }
        }
        Ok(ConfigSnapshot::new(files))
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

fn is_hcl_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("piing_hcl")
    )
}

impl ConfigPaths {
    fn has_hcl_files(&self) -> Result<bool> {
        if !self.config_dir.exists() {
            return Ok(false);
        }
        for entry in fs::read_dir(&self.config_dir)? {
            let entry = entry?;
            if is_hcl_file(&entry.path()) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

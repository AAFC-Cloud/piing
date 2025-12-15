use crate::config::ConfigPaths;
use crate::config::ConfigSnapshot;
use crate::config::ConfigStore;
use eyre::Result;

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub paths: ConfigPaths,
    pub store: ConfigStore,
}

impl ConfigManager {
    /// # Errors
    /// Returns an error if config initialization fails
    pub fn initialize() -> Result<Self> {
        let paths = ConfigPaths::new();
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

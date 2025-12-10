use crate::config::ConfigSnapshot;
use std::sync::Arc;
use std::sync::RwLock;

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

use crate::config::targets::Target;
use crate::config::targets::decode_targets;
use crate::config::vpn::VpnCriterion;
use crate::config::vpn::decode_vpn_criteria;
use eyre::Result;
use hcl::edit::structure::Body;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub files: BTreeMap<PathBuf, Body>,
    decoded_targets: Option<Vec<Target>>,
    decoded_vpn_criteria: Option<Vec<VpnCriterion>>,
}

impl ConfigSnapshot {
    #[must_use]
    pub fn new(files: BTreeMap<PathBuf, Body>) -> Self {
        Self {
            files,
            decoded_targets: None,
            decoded_vpn_criteria: None,
        }
    }

    /// # Errors
    /// Returns an error if parsing target resources fails
    pub fn targets(&mut self) -> Result<&[Target]> {
        if self.decoded_targets.is_none() {
            let mut decoded = Vec::new();
            for (path, body) in &self.files {
                decoded.extend(decode_targets(path, body)?);
            }
            self.decoded_targets = Some(decoded);
        }
        Ok(self.decoded_targets.as_deref().unwrap_or(&[]))
    }

    /// # Errors
    /// Returns an error if parsing VPN resources fails
    pub fn vpn_criteria(&mut self) -> Result<&[VpnCriterion]> {
        if self.decoded_vpn_criteria.is_none() {
            let mut decoded = Vec::new();
            for (path, body) in &self.files {
                decoded.extend(decode_vpn_criteria(path, body)?);
            }
            self.decoded_vpn_criteria = Some(decoded);
        }
        Ok(self.decoded_vpn_criteria.as_deref().unwrap_or(&[]))
    }
}

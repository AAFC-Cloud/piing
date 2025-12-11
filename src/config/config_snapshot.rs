use crate::config::targets::Target;
use crate::config::targets::decode_targets;
use crate::config::vpn_criterion::VpnCriterion;
use crate::config::vpn_criterion::decode_vpn_criteria;
use eyre::Context as _;
use eyre::Result;
use hcl::edit::parser::parse_body;
use hcl::edit::structure::Body;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub files: BTreeMap<PathBuf, Body>,
    pub targets: Vec<Target>,
    pub vpn_criteria: Vec<VpnCriterion>,
}

impl ConfigSnapshot {
    #[must_use]
    pub fn new(
        files: BTreeMap<PathBuf, Body>,
        targets: Vec<Target>,
        vpn_criteria: Vec<VpnCriterion>,
    ) -> Self {
        Self {
            files,
            targets,
            vpn_criteria,
        }
    }

    #[must_use]
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    #[must_use]
    pub fn vpn_criteria(&self) -> &[VpnCriterion] {
        &self.vpn_criteria
    }

    /// # Errors
    /// Returns an error if reading or parsing any config file fails
    pub fn try_from_dir(dir: &Path) -> Result<Self> {
        let mut files = BTreeMap::new();
        let mut targets = Vec::new();
        let mut vpn_criteria = Vec::new();

        if dir.exists() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if !is_hcl_file(&path) {
                    continue;
                }
                let content = fs::read_to_string(&path)
                    .wrap_err_with(|| format!("Failed to read config file: {}", path.display()))?;
                let body: Body = parse_body(&content)
                    .wrap_err_with(|| format!("Failed to parse config file: {}", path.display()))?;
                targets.extend(decode_targets(&path, &body)?);
                vpn_criteria.extend(decode_vpn_criteria(&path, &body)?);
                files.insert(path, body);
            }
        }

        Ok(Self::new(files, targets, vpn_criteria))
    }
}

fn is_hcl_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("piing_hcl")
    )
}

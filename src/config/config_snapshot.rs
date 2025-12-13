use crate::config::log_latency::LatencyColouration;
use crate::config::log_latency::build_latency_body;
use crate::config::log_latency::decode_latency_coloration;
use crate::config::targets::Target;
use crate::config::targets::decode_targets;
use crate::config::vpn_criterion::VpnCriterion;
use crate::config::vpn_criterion::decode_vpn_criteria;
use chrono::Utc;
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
    pub latency_colouration: LatencyColouration,
}

impl ConfigSnapshot {
    #[must_use]
    pub fn new(
        files: BTreeMap<PathBuf, Body>,
        targets: Vec<Target>,
        vpn_criteria: Vec<VpnCriterion>,
        latency_colouration: LatencyColouration,
    ) -> Self {
        Self {
            files,
            targets,
            vpn_criteria,
            latency_colouration,
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

    #[must_use]
    pub fn latency_colouration(&self) -> &LatencyColouration {
        &self.latency_colouration
    }

    /// # Errors
    /// Returns an error if reading or parsing any config file fails
    pub fn try_from_dir(dir: &Path) -> Result<Self> {
        let mut files = BTreeMap::new();
        let mut targets = Vec::new();
        let mut vpn_criteria = Vec::new();
        let mut latency_rules = Vec::new();
        let mut latency_rules_found = false;

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
                let mut decoded = decode_latency_coloration(&path, &body)?;
                if !decoded.is_empty() {
                    latency_rules_found = true;
                }
                latency_rules.append(&mut decoded);
                files.insert(path, body);
            }
        }

        let latency_colouration = LatencyColouration::from_rules_or_default(latency_rules);

        if !latency_rules_found {
            let body = build_latency_body("latency_defaults", &latency_colouration);
            let file_path = unique_file_path(dir, "latency");
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, body.to_string()).wrap_err_with(|| {
                format!(
                    "Failed to write default latency colouration config to {}",
                    file_path.display()
                )
            })?;
            files.insert(file_path, body);
        }

        Ok(Self::new(files, targets, vpn_criteria, latency_colouration))
    }
}

fn unique_file_path(dir: &Path, stem: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y-%m-%d_%H%M%S");
    let mut candidate = format!("{timestamp}_{stem}.piing_hcl");
    let mut counter = 1;
    loop {
        let path = dir.join(&candidate);
        if !path.exists() {
            return path;
        }
        counter += 1;
        candidate = format!("{timestamp}_{stem}_{counter}.piing_hcl");
    }
}

fn is_hcl_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("piing_hcl")
    )
}

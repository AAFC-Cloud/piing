use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use teamy_windows::network::NetworkAdapterExt;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnCriterion {
    pub block_label: String,
    pub properties: VpnCriterionProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnCriterionProperties {
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VpnCriteria(pub Vec<VpnCriterion>);

impl From<IP_ADAPTER_ADDRESSES_LH> for VpnCriterion {
    fn from(adapter: IP_ADAPTER_ADDRESSES_LH) -> Self {
        let name = adapter.display_name().to_string();
        let properties = VpnCriterionProperties {
            display_name: Some(name.clone()),
        };
        Self { block_label: name, properties }
    }
}

impl VpnCriterionProperties {
    pub fn matches(&self, adapter: &IP_ADAPTER_ADDRESSES_LH) -> bool {
        if let Some(ref display_name) = self.display_name {
            display_name == &adapter.display_name()
        } else {
            false
        }
    }
}

impl VpnCriterion {
    pub fn matches(&self, adapter: &IP_ADAPTER_ADDRESSES_LH) -> bool {
        self.properties.matches(adapter)
    }
}

impl VpnCriteria {
    /// Read the `.piing_hcl` files in the dir to parse the rules
    pub fn try_from_dir(dir_path: impl AsRef<Path>) -> Result<Self> {
        let dir = dir_path.as_ref();
        let mut criteria = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("piing_hcl") {
                let content = std::fs::read_to_string(&path)?;
                let value: hcl::Value = hcl::from_str(&content)?;
                if let hcl::Value::Object(map) = value {
                    if let Some(hcl::Value::Object(resource_map)) = map.get("resource") {
                        if let Some(hcl::Value::Object(type_map)) = resource_map.get("piing_vpncriterion") {
                            for (name, props_value) in type_map {
                                if let hcl::Value::Object(props_map) = props_value {
                                    let display_name = props_map.get("display_name").and_then(|v| v.as_str()).map(|s| s.to_string());
                                    let properties = VpnCriterionProperties { display_name };
                                    let criterion = VpnCriterion { block_label: name.clone(), properties };
                                    criteria.push(criterion);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(Self(criteria))
    }

    /// Write the rules to the dir, which must be empty
    pub fn try_write_dir(&self, dir_path: impl AsRef<Path>) -> Result<()> {
        let dir = dir_path.as_ref();
        if std::fs::read_dir(dir)?.next().is_some() {
            return Err(eyre!("Directory is not empty"));
        }
        for criterion in &self.0 {
            let mut props_map = hcl::Map::new();
            if let Some(display_name) = &criterion.properties.display_name {
                props_map.insert("display_name".to_string(), hcl::Value::String(display_name.clone()));
            }
            let mut type_map = hcl::Map::new();
            type_map.insert(criterion.block_label.clone(), hcl::Value::Object(props_map));
            let mut resource_map = hcl::Map::new();
            resource_map.insert("piing_vpncriterion".to_string(), hcl::Value::Object(type_map));
            let mut map = hcl::Map::new();
            map.insert("resource".to_string(), hcl::Value::Object(resource_map));
            let value = hcl::Value::Object(map);
            let content = hcl::to_string(&value)?;
            let filename = format!("{}.piing_hcl", criterion.block_label);
            std::fs::write(dir.join(filename), content)?;
        }
        Ok(())
    }

    /// Write the rules to the dir, prompting to wipe if not empty
    pub fn write_dir_interactive(&self, dir_path: impl AsRef<Path>) -> Result<()> {
        let dir = dir_path.as_ref();
        if std::fs::read_dir(dir)?.next().is_some() {
            println!("Directory is not empty. Wipe it? (y/N)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                return Err(eyre!("User declined to wipe directory"));
            }
            // wipe
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }
        self.try_write_dir(dir_path)
    }

    /// Write the criteria to a single file
    pub fn write_to_file(&self, file_path: impl AsRef<Path>) -> Result<()> {
        let mut type_map = hcl::Map::new();
        for criterion in &self.0 {
            let mut props_map = hcl::Map::new();
            if let Some(display_name) = &criterion.properties.display_name {
                props_map.insert("display_name".to_string(), hcl::Value::String(display_name.clone()));
            }
            type_map.insert(criterion.block_label.clone(), hcl::Value::Object(props_map));
        }
        let mut resource_map = hcl::Map::new();
        resource_map.insert("piing_vpncriterion".to_string(), hcl::Value::Object(type_map));
        let mut map = hcl::Map::new();
        map.insert("resource".to_string(), hcl::Value::Object(resource_map));
        let value = hcl::Value::Object(map);
        let content = hcl::to_string(&value)?;
        std::fs::write(file_path, content)?;
        Ok(())
    }
}
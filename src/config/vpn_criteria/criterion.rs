use super::VpnCriterionProperties;
use eyre::bail;
use hcl::edit::Ident;
use hcl::edit::structure::Block;
use serde::Deserialize;
use serde::Serialize;
use teamy_windows::network::NetworkAdapterExt;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnCriterion {
    pub name: String,
    pub properties: VpnCriterionProperties,
}

impl From<IP_ADAPTER_ADDRESSES_LH> for VpnCriterion {
    fn from(adapter: IP_ADAPTER_ADDRESSES_LH) -> Self {
        let name = adapter.display_name().to_string();
        let properties = VpnCriterionProperties {
            display_name: Some(name.clone()),
        };
        Self { name, properties }
    }
}

impl VpnCriterion {
    pub fn matches(&self, adapter: &IP_ADAPTER_ADDRESSES_LH) -> bool {
        self.properties.matches(adapter)
    }
}

impl TryFrom<Block> for VpnCriterion {
    type Error = eyre::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        if block.ident.as_str() != "resource" {
            bail!("Invalid block ident: {}", block.ident);
        }
        let [kind, name] = block.labels.as_slice() else {
            bail!("Invalid block labels: {:?}", block.labels);
        };
        if kind.as_str() != "piing_vpncriterion" {
            bail!("Invalid resource type: {:?}", kind);
        }
        let properties = block.body.try_into()?;
        Ok(Self {
            name: name.as_str().to_string(),
            properties,
        })
    }
}

impl From<VpnCriterion> for Block {
    fn from(criterion: VpnCriterion) -> Self {
        let mut block = Block::builder(Ident::new("resource"))
            .label("piing_vpncriterion")
            .label(criterion.name.as_str())
            .build();
        block.body = criterion.properties.into();
        block
    }
}

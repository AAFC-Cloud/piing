use eyre::Result;
use hcl::edit::Decorated;
use hcl::edit::Ident;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::path::Path;
use teamy_windows::network::NetworkAdapterExt;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;

#[derive(Debug, Clone)]
pub struct VpnCriterion {
    pub display_name: Option<String>,
}

impl VpnCriterion {
    #[must_use]
    pub fn matches(&self, adapter: &IP_ADAPTER_ADDRESSES_LH) -> bool {
        if let Some(ref display_name) = self.display_name {
            display_name == &adapter.display_name()
        } else {
            false
        }
    }

    #[must_use]
    pub fn block(&self, name: &str) -> Block {
        let mut block = Block::builder(Ident::new("resource"))
            .label("piing_vpn_criterion")
            .label(name)
            .build();
        block.body = self.clone().into();
        block
    }
}

impl From<IP_ADAPTER_ADDRESSES_LH> for VpnCriterion {
    fn from(adapter: IP_ADAPTER_ADDRESSES_LH) -> Self {
        Self {
            display_name: Some(adapter.display_name().to_string()),
        }
    }
}

/// # Errors
/// Returns an error if a block cannot be decoded into a `VpnCriterion`
pub fn decode_vpn_criteria(file_path: &Path, body: &Body) -> Result<Vec<VpnCriterion>> {
    let mut criteria = Vec::new();
    for structure in body.clone() {
        let Structure::Block(block) = structure else {
            continue;
        };
        let Some((resource_type, _name)) = block.labels.split_first() else {
            continue;
        };
        if resource_type.as_str() != "piing_vpn_criterion" {
            continue;
        }
        criteria.push(block.body.clone().try_into().map_err(|error| {
            eyre::eyre!(
                "Failed to decode VPN criterion in {}: {error}",
                file_path.display()
            )
        })?);
    }
    Ok(criteria)
}

impl TryFrom<Body> for VpnCriterion {
    type Error = eyre::Error;

    fn try_from(body: Body) -> Result<Self, Self::Error> {
        let display_name: Option<String> = match body.get_attribute("display_name") {
            Some(attr) => Some(
                attr.value
                    .as_str()
                    .ok_or_else(|| eyre::eyre!("Invalid display_name value"))?
                    .to_string(),
            ),
            None => None,
        };
        Ok(Self { display_name })
    }
}

impl From<VpnCriterion> for Body {
    fn from(props: VpnCriterion) -> Self {
        let mut body = Body::builder();
        if let Some(display_name) = props.display_name {
            body = body.attribute(Attribute::new(
                Decorated::new(Ident::new("display_name")),
                Expression::String(Decorated::new(display_name)),
            ));
        }
        body.build()
    }
}

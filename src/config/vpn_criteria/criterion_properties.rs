use hcl::edit::Decorate;
use hcl::edit::Decorated;
use hcl::edit::Ident;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Body;
use serde::Deserialize;
use serde::Serialize;
use teamy_windows::network::NetworkAdapterExt;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnCriterionProperties {
    pub display_name: Option<String>,
}

impl VpnCriterionProperties {
    #[must_use]
    pub fn matches(&self, adapter: &IP_ADAPTER_ADDRESSES_LH) -> bool {
        if let Some(ref display_name) = self.display_name {
            display_name == &adapter.display_name()
        } else {
            false
        }
    }
}

impl TryFrom<Body> for VpnCriterionProperties {
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

impl From<VpnCriterionProperties> for Body {
    fn from(props: VpnCriterionProperties) -> Self {
        let mut body = Body::builder();
        if let Some(display_name) = props.display_name {
            body = body.attribute(hcl::edit::structure::Attribute::new(
                Decorated::new(Ident::new("display_name")).decorated(("  ", " ")),
                Expression::String(Decorated::new(display_name)),
            ));
        }
        body.build()
    }
}

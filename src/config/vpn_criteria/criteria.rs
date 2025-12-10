use super::VpnCriterion;
use eyre::Result;
use eyre::bail;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct VpnCriteria(pub Vec<VpnCriterion>);

impl VpnCriteria {
    /// Read the `.piing_hcl` files in the dir to parse the rules
    ///
    /// # Errors
    ///
    /// Returns an error if reading, parsing, or converting any
    /// `.piing_hcl` file in the directory fails.
    pub fn try_from_dir(dir_path: impl AsRef<Path>) -> Result<Self> {
        let dir = dir_path.as_ref();
        let mut criteria = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("piing_hcl") {
                let content = std::fs::read_to_string(&path)?;
                let body = hcl::edit::parser::parse_body(&content)?;
                for structure in body {
                    let Structure::Block(block) = structure else {
                        bail!(
                            "Expected only blocks block in VPN criterion file: {:?}, found unrecognized structure: {}",
                            path,
                            Body::builder().structure(structure).build().to_string()
                        );
                    };
                    let criterion = VpnCriterion::try_from(block)?;
                    criteria.push(criterion);
                }
            }
        }
        Ok(Self(criteria))
    }

    /// Write the criteria to a single file
    ///
    /// # Errors
    ///
    /// Returns an error if serializing the criteria or writing the
    /// resulting content to `file_path` fails.
    pub fn write_to_file(&self, file_path: impl AsRef<Path>) -> Result<()> {
        let mut body = Body::builder();
        for criterion in &self.0 {
            let block = Block::from(criterion.clone());
            body = body.block(block);
        }
        let body = body.build();
        let content = body.to_string();
        std::fs::write(file_path, content)?;
        Ok(())
    }
}

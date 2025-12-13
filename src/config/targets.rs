use crate::ping::PingMode;
use eyre::Context as _;
use eyre::Result;
use hcl::edit::Decorate;
use hcl::edit::Decorated;
use hcl::edit::Ident;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId {
    pub file_path: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub id: TargetId,
    pub value: String,
    pub mode: PingMode,
    pub interval: Duration,
}

impl Target {
    #[must_use]
    pub fn block(&self) -> Block {
        build_block(&self.id.name, &self.value, self.mode, self.interval)
    }
}

#[must_use]
pub fn build_block(name: &str, value: &str, mode: PingMode, interval: Duration) -> Block {
    let mut block = Block::builder(Ident::new("resource"))
        .label("piing_target")
        .label(name)
        .build();
    block.body = build_body(value, mode, interval);
    block
}

fn build_body(value: &str, mode: PingMode, interval: Duration) -> Body {
    let mut body = Body::builder();
    body = body.attribute(Attribute::new(
        Decorated::new(Ident::new("value")).decorated(("  ", " ")),
        Expression::String(Decorated::new(value.to_string())),
    ));
    body = body.attribute(Attribute::new(
        Decorated::new(Ident::new("mode")).decorated(("  ", " ")),
        Expression::String(Decorated::new(mode.as_str().to_string())),
    ));
    body = body.attribute(Attribute::new(
        Decorated::new(Ident::new("interval")).decorated(("  ", " ")),
        Expression::String(Decorated::new(
            humantime::format_duration(interval).to_string(),
        )),
    ));
    body.build()
}

/// # Errors
/// Returns an error if required attributes are missing or invalid types/values are encountered
pub fn decode_targets(file_path: &Path, body: &Body) -> Result<Vec<Target>> {
    let mut targets = Vec::new();
    for structure in body.clone() {
        let Structure::Block(block) = structure else {
            continue;
        };
        let mut labels = block.labels.iter();
        let Some(resource_type) = labels.next() else {
            continue;
        };
        let Some(name_ident) = labels.next() else {
            continue;
        };
        // todo: ensure no more labels
        if resource_type.as_str() != "piing_target" {
            continue;
        }
        let name = name_ident.as_str().to_string();
        let value = read_string_attribute(&block, "value", file_path, &name)?;
        let mode_str = read_string_attribute(&block, "mode", file_path, &name)?;
        let mode = <PingMode as PingModeParseExt>::from_str_case_insensitive(&mode_str)
            .wrap_err_with(|| format!("Invalid mode in {} -> {name}", file_path.display()))?;
        let interval_raw = read_string_attribute(&block, "interval", file_path, &name)?;
        let interval = humantime::parse_duration(&interval_raw).wrap_err_with(|| {
            format!(
                "Invalid interval '{}' in {} -> {name}",
                interval_raw,
                file_path.display()
            )
        })?;
        targets.push(Target {
            id: TargetId {
                file_path: file_path.to_path_buf(),
                name,
            },
            value,
            mode,
            interval,
        });
    }
    Ok(targets)
}

fn read_string_attribute(block: &Block, key: &str, file_path: &Path, name: &str) -> Result<String> {
    let attribute = block.body.get_attribute(key).ok_or_else(|| {
        eyre::eyre!(
            "Missing attribute '{}' in {} -> {}",
            key,
            file_path.display(),
            name
        )
    })?;
    attribute
        .value
        .as_str()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| {
            eyre::eyre!(
                "Attribute '{}' must be a string in {} -> {}",
                key,
                file_path.display(),
                name
            )
        })
}

#[must_use]
pub fn sanitize_label(input: &str) -> String {
    let mut sanitized = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            sanitized.push(ch);
        } else if ch.is_whitespace() {
            sanitized.push('_');
        }
    }
    if sanitized.is_empty() {
        sanitized.push('t');
    }
    sanitized
}

/// Provide manual case-insensitive parsing helper while keeping Clap's [`ValueEnum`] API.
trait PingModeParseExt {
    fn from_str_case_insensitive(value: &str) -> Result<PingMode>;
}

impl PingModeParseExt for PingMode {
    fn from_str_case_insensitive(value: &str) -> Result<PingMode> {
        match value.to_ascii_lowercase().as_str() {
            "icmp" => Ok(PingMode::Icmp),
            "tcp" => Ok(PingMode::Tcp),
            "http-get" | "http_get" => Ok(PingMode::HttpGet),
            "http-head" | "http_head" => Ok(PingMode::HttpHead),
            other => Err(eyre::eyre!("Unsupported mode '{other}'")),
        }
    }
}

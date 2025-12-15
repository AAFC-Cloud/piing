use eyre::Context as _;
use eyre::Result;
use hcl::edit::Decorated;
use hcl::edit::Ident;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::path::Path;
use std::path::PathBuf;

pub const DEFAULT_PROBLEM_SOUND_PATH: &str = r"C:\Windows\Media\Speech Off.wav";
const DEFAULT_VOLUME: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct ProblemSound {
    path: PathBuf,
    volume: f32,
}

impl ProblemSound {
    #[must_use]
    pub fn new(path: PathBuf, volume: f32) -> Self {
        Self { path, volume }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn volume(&self) -> f32 {
        self.volume
    }
}

/// Decode at most one problem sound block from the provided body.
///
/// # Errors
/// Returns an error if more than one block is present, required attributes are
/// missing, values are invalid, or the referenced file does not exist.
pub fn decode_problem_sound(file_path: &Path, body: &Body) -> Result<Option<ProblemSound>> {
    let mut sound: Option<ProblemSound> = None;

    for structure in body {
        let Structure::Block(block) = structure else {
            continue;
        };
        let mut labels = block.labels.iter();
        let Some(resource_type) = labels.next() else {
            continue;
        };
        if resource_type.as_str() != "piing_problem_sound" {
            continue;
        }

        // Enforce 0-1 blocks across all files.
        if sound.is_some() {
            return Err(eyre::eyre!(
                "Multiple piing_problem_sound blocks found; only one is allowed"
            ));
        }

        let path_attr = block.body.get_attribute("path").ok_or_else(|| {
            eyre::eyre!(
                "Missing 'path' attribute in piing_problem_sound block inside {}",
                file_path.display()
            )
        })?;
        let path_str = path_attr
            .value
            .as_str()
            .ok_or_else(|| {
                eyre::eyre!(
                    "Attribute 'path' must be a string in piing_problem_sound block inside {}",
                    file_path.display()
                )
            })?
            .to_string();
        let path = PathBuf::from(path_str);
        validate_sound_path(&path)?;

        let volume = match block.body.get_attribute("volume") {
            Some(attr) => {
                // Accept numeric or string values for volume so builders can emit either.
                let raw_num = attr.value.as_number().and_then(hcl::Number::as_f64);
                let raw_from_str = attr.value.as_str().and_then(|s| s.parse::<f64>().ok());
                let raw = raw_num.or(raw_from_str);
                parse_volume(raw).wrap_err_with(|| {
                    format!(
                        "Invalid 'volume' value in piing_problem_sound block inside {}",
                        file_path.display()
                    )
                })?
            }
            None => DEFAULT_VOLUME,
        };

        sound = Some(ProblemSound::new(path, volume));
    }

    Ok(sound)
}

/// Build a default `piing_problem_sound` resource body with the provided name,
/// `path`, and `volume`.
#[must_use]
pub fn build_problem_sound_body(name: &str, path: &Path, volume: f32) -> Body {
    let mut block = Block::builder(Ident::new("resource"))
        .label("piing_problem_sound")
        .label(name)
        .build();

    let mut body = Body::builder();
    body = body.attribute(Attribute::new(
        Decorated::new(Ident::new("path")),
        Expression::String(Decorated::new(path.to_string_lossy().to_string())),
    ));
    // Emit volume as a string to match other builders, but parsing accepts both
    // string and numeric forms.
    body = body.attribute(Attribute::new(
        Decorated::new(Ident::new("volume")),
        Expression::String(Decorated::new(volume.to_string())),
    ));

    block.body = body.build();
    Body::builder().block(block).build()
}

fn validate_sound_path(path: &Path) -> Result<()> {
    let metadata = path
        .metadata()
        .wrap_err_with(|| format!("Problem sound path does not exist: {}", path.display()))?;
    if !metadata.is_file() {
        return Err(eyre::eyre!(
            "Problem sound path is not a file: {}",
            path.display()
        ));
    }
    Ok(())
}

fn parse_volume(raw: Option<f64>) -> Result<f32> {
    let Some(raw) = raw else {
        return Err(eyre::eyre!("Volume must be a number between 0.0 and 1.0"));
    };
    #[allow(clippy::cast_possible_truncation)]
    let volume = raw as f32;
    if !(0.0..=1.0).contains(&volume) {
        return Err(eyre::eyre!(
            "Volume must be between 0.0 and 1.0 (inclusive); received {volume}"
        ));
    }
    Ok(volume)
}

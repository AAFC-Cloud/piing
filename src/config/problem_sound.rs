use eyre::Context as _;
use eyre::Result;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::path::Path;
use std::path::PathBuf;

pub const DEFAULT_PROBLEM_SOUND_PATH: &str = r"C:\\Windows\\Media\\Speech Off.wav";
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
            Some(attr) => parse_volume(attr.value.as_number().and_then(hcl::Number::as_f64))
                .wrap_err_with(|| {
                    format!(
                        "Invalid 'volume' value in piing_problem_sound block inside {}",
                        file_path.display()
                    )
                })?,
            None => DEFAULT_VOLUME,
        };

        sound = Some(ProblemSound::new(path, volume));
    }

    Ok(sound)
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

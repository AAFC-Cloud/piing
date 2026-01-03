use eyre::Context as _;
use eyre::Result;
use hcl::edit::Decorated;
use hcl::edit::Ident;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

pub const DEFAULT_PROBLEM_SOUND_PATH: &str = r"C:\Windows\Media\Speech Off.wav";
const DEFAULT_VOLUME: f32 = 1.0;

/// Controls when the problem sound is played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SoundMode {
    /// Never play the sound.
    Never,
    /// Always play the sound when a problem is detected.
    #[default]
    Always,
    /// Play the sound only when not connected to a VPN.
    NotOnVpn,
}

impl SoundMode {
    /// Returns the HCL attribute value for this mode.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Always => "always",
            Self::NotOnVpn => "not_vpn",
        }
    }
}

impl fmt::Display for SoundMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SoundMode {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(Self::Never),
            "always" => Ok(Self::Always),
            "not_vpn" => Ok(Self::NotOnVpn),
            _ => Err(eyre::eyre!(
                "Invalid sound mode '{s}'; expected 'never', 'always', or 'not_vpn'"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProblemSound {
    path: PathBuf,
    volume: f32,
    mode: SoundMode,
    /// The config file where this problem sound is defined, if known.
    source_file: Option<PathBuf>,
}

impl ProblemSound {
    #[must_use]
    pub fn new(path: PathBuf, volume: f32) -> Self {
        Self {
            path,
            volume,
            mode: SoundMode::default(),
            source_file: None,
        }
    }

    #[must_use]
    pub fn with_mode(path: PathBuf, volume: f32, mode: SoundMode) -> Self {
        Self {
            path,
            volume,
            mode,
            source_file: None,
        }
    }

    #[must_use]
    pub fn with_source(mut self, source_file: PathBuf) -> Self {
        self.source_file = Some(source_file);
        self
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn volume(&self) -> f32 {
        self.volume
    }

    #[must_use]
    pub fn mode(&self) -> SoundMode {
        self.mode
    }

    #[must_use]
    pub fn source_file(&self) -> Option<&Path> {
        self.source_file.as_deref()
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

        let mode = match block.body.get_attribute("when") {
            Some(attr) => {
                let mode_str = attr.value.as_str().ok_or_else(|| {
                    eyre::eyre!(
                        "Attribute 'when' must be a string in piing_problem_sound block inside {}",
                        file_path.display()
                    )
                })?;
                mode_str.parse::<SoundMode>().wrap_err_with(|| {
                    format!(
                        "Invalid 'when' value in piing_problem_sound block inside {}",
                        file_path.display()
                    )
                })?
            }
            None => SoundMode::default(),
        };

        sound = Some(
            ProblemSound::with_mode(path, volume, mode).with_source(file_path.to_path_buf()),
        );
    }

    Ok(sound)
}

/// Build a default `piing_problem_sound` resource body with the provided name,
/// `path`, `volume`, and `mode`.
#[must_use]
pub fn build_problem_sound_body(name: &str, path: &Path, volume: f32, mode: SoundMode) -> Body {
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
    body = body.attribute(Attribute::new(
        Decorated::new(Ident::new("when")),
        Expression::String(Decorated::new(mode.to_string())),
    ));

    block.body = body.build();
    Body::builder().block(block).build()
}

/// Update the `when` attribute in the problem sound config file.
///
/// This function reads the config file, finds the `piing_problem_sound` block,
/// updates the `when` attribute, and writes the file back.
///
/// # Errors
/// Returns an error if the source file is unknown, the file cannot be read/written,
/// or the HCL cannot be parsed.
pub fn update_sound_mode(sound: &ProblemSound, new_mode: SoundMode) -> Result<()> {
    let source_file = sound
        .source_file()
        .ok_or_else(|| eyre::eyre!("Cannot update sound mode: source file unknown"))?;

    let content = fs::read_to_string(source_file)
        .wrap_err_with(|| format!("Failed to read config file: {}", source_file.display()))?;

    let mut body: Body = hcl::edit::parser::parse_body(&content)
        .wrap_err_with(|| format!("Failed to parse config file: {}", source_file.display()))?;

    // Find and update the piing_problem_sound block
    let mut found = false;
    for mut structure in &mut body {
        let Some(block) = structure.as_block_mut() else {
            continue;
        };
        let mut labels = block.labels.iter();
        let Some(resource_type) = labels.next() else {
            continue;
        };
        if resource_type.as_str() != "piing_problem_sound" {
            continue;
        }

        // Remove existing 'when' attribute if present, then push the new one
        block.body.remove_attribute("when");
        block.body.push(Attribute::new(
            Decorated::new(Ident::new("when")),
            Expression::String(Decorated::new(new_mode.to_string())),
        ));
        found = true;
        break;
    }

    if !found {
        return Err(eyre::eyre!(
            "No piing_problem_sound block found in {}",
            source_file.display()
        ));
    }

    fs::write(source_file, body.to_string())
        .wrap_err_with(|| format!("Failed to write config file: {}", source_file.display()))?;

    Ok(())
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

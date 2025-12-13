use eyre::Result;
use hcl::edit::Decorated;
use hcl::edit::Ident;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use owo_colors::AnsiColors;
use std::cmp::Ordering;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum LatencyColour {
    Ansi(AnsiColors),
}

impl LatencyColour {
    /// # Errors
    /// Returns an error if the colour name is not a supported ANSI colour
    pub fn parse(name: &str) -> Result<Self> {
        let normalized = name.to_ascii_lowercase();
        let colour = match normalized.as_str() {
            "black" => AnsiColors::Black,
            "red" => AnsiColors::Red,
            "green" => AnsiColors::Green,
            "yellow" => AnsiColors::Yellow,
            "blue" => AnsiColors::Blue,
            "magenta" | "purple" => AnsiColors::Magenta,
            "cyan" | "teal" => AnsiColors::Cyan,
            "white" => AnsiColors::White,
            "bright_black" | "bright-black" | "gray" | "grey" => AnsiColors::BrightBlack,
            "bright_red" | "bright-red" => AnsiColors::BrightRed,
            "bright_green" | "bright-green" => AnsiColors::BrightGreen,
            "bright_yellow" | "bright-yellow" => AnsiColors::BrightYellow,
            "bright_blue" | "bright-blue" => AnsiColors::BrightBlue,
            "bright_magenta" | "bright-magenta" => AnsiColors::BrightMagenta,
            "bright_cyan" | "bright-cyan" => AnsiColors::BrightCyan,
            "bright_white" | "bright-white" => AnsiColors::BrightWhite,
            other => {
                return Err(eyre::eyre!(
                    "Unsupported foreground_color '{other}', expected a basic ANSI colour name"
                ));
            }
        };
        Ok(Self::Ansi(colour))
    }

    #[must_use]
    pub fn to_ansi(self) -> AnsiColors {
        match self {
            Self::Ansi(colour) => colour,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatencyRule {
    pub below_ms: f64,
    pub foreground: LatencyColour,
}

#[derive(Debug, Clone)]
pub struct LatencyColouration {
    rules: Vec<LatencyRule>,
}

impl LatencyColouration {
    #[must_use]
    pub fn from_rules_or_default(mut rules: Vec<LatencyRule>) -> Self {
        if rules.is_empty() {
            return Self::default();
        }
        rules.sort_by(|a, b| match a.below_ms.partial_cmp(&b.below_ms) {
            Some(ordering) => ordering,
            None => Ordering::Equal,
        });
        Self { rules }
    }

    #[must_use]
    pub fn color_for(&self, latency_ms: f64) -> Option<AnsiColors> {
        for rule in &self.rules {
            if latency_ms < rule.below_ms {
                return Some(rule.foreground.to_ansi());
            }
        }
        self.rules.last().map(|rule| rule.foreground.to_ansi())
    }

    #[must_use]
    pub fn rules(&self) -> &[LatencyRule] {
        &self.rules
    }
}

impl Default for LatencyColouration {
    fn default() -> Self {
        let rules = vec![
            LatencyRule {
                below_ms: 20.0,
                foreground: LatencyColour::Ansi(AnsiColors::Green),
            },
            LatencyRule {
                below_ms: 100.0,
                foreground: LatencyColour::Ansi(AnsiColors::Yellow),
            },
            LatencyRule {
                below_ms: 9_999.0,
                foreground: LatencyColour::Ansi(AnsiColors::Red),
            },
        ];
        Self { rules }
    }
}

/// Decode all latency colouration rules found in a config file body.
///
/// # Errors
/// Returns an error if attributes are missing or malformed
pub fn decode_latency_coloration(file_path: &Path, body: &Body) -> Result<Vec<LatencyRule>> {
    let mut rules = Vec::new();
    for structure in body.clone() {
        let Structure::Block(block) = structure else {
            continue;
        };
        let mut labels = block.labels.iter();
        let Some(resource_type) = labels.next() else {
            continue;
        };
        if resource_type.as_str() != "piing_log_latency_colouration" {
            continue;
        }

        for structure in block.body.clone() {
            let Structure::Block(when_block) = structure else {
                continue;
            };
            if when_block.ident.as_str() != "when" {
                continue;
            }

            let below_attr = when_block.body.get_attribute("below").ok_or_else(|| {
                eyre::eyre!(
                    "Missing 'below' attribute in latency colouration block inside {}",
                    file_path.display()
                )
            })?;
            let below_ms = parse_number(&below_attr.value).ok_or_else(|| {
                eyre::eyre!(
                    "Invalid 'below' value in latency colouration block inside {}",
                    file_path.display()
                )
            })?;

            let fg_attr = when_block
                .body
                .get_attribute("foreground_color")
                .ok_or_else(|| {
                    eyre::eyre!(
                        "Missing 'foreground_color' attribute in latency colouration block inside {}",
                        file_path.display()
                    )
                })?;
            let colour_name = fg_attr
                .value
                .as_str()
                .ok_or_else(|| {
                    eyre::eyre!(
                        "foreground_color must be a string in latency colouration block inside {}",
                        file_path.display()
                    )
                })?
                .to_string();
            let foreground = LatencyColour::parse(&colour_name).map_err(|error| {
                eyre::eyre!(
                    "Failed to parse foreground_color in {}: {error}",
                    file_path.display()
                )
            })?;

            rules.push(LatencyRule {
                below_ms,
                foreground,
            });
        }
    }

    Ok(rules)
}

fn parse_number(value: &Expression) -> Option<f64> {
    if let Some(num) = value.as_number().and_then(hcl::Number::as_f64) {
        return Some(num);
    }
    value.as_str().and_then(|s| s.parse::<f64>().ok())
}

#[must_use]
pub fn build_latency_block(name: &str, colouration: &LatencyColouration) -> Block {
    let mut block = Block::builder(Ident::new("resource"))
        .label("piing_log_latency_colouration")
        .label(name)
        .build();

    let mut body = Body::builder();
    for rule in colouration.rules() {
        let mut when_block = Block::builder(Ident::new("when")).build();
        let mut when_body = Body::builder();
        when_body = when_body.attribute(Attribute::new(
            Decorated::new(Ident::new("below")),
            Expression::String(Decorated::new(rule.below_ms.to_string())),
        ));
        when_body = when_body.attribute(Attribute::new(
            Decorated::new(Ident::new("foreground_color")),
            Expression::String(Decorated::new(format_colour(rule.foreground).to_string())),
        ));
        when_block.body = when_body.build();
        body = body.block(when_block);
    }

    block.body = body.build();
    block
}

#[must_use]
pub fn build_latency_body(name: &str, colouration: &LatencyColouration) -> Body {
    Body::builder()
        .block(build_latency_block(name, colouration))
        .build()
}

fn format_colour(colour: LatencyColour) -> &'static str {
    match colour {
        LatencyColour::Ansi(AnsiColors::Black) => "black",
        LatencyColour::Ansi(AnsiColors::Red) => "red",
        LatencyColour::Ansi(AnsiColors::Green) => "green",
        LatencyColour::Ansi(AnsiColors::Yellow) => "yellow",
        LatencyColour::Ansi(AnsiColors::Blue) => "blue",
        LatencyColour::Ansi(AnsiColors::Magenta) => "magenta",
        LatencyColour::Ansi(AnsiColors::Cyan) => "cyan",
        LatencyColour::Ansi(AnsiColors::BrightBlack) => "bright_black",
        LatencyColour::Ansi(AnsiColors::BrightRed) => "bright_red",
        LatencyColour::Ansi(AnsiColors::BrightGreen) => "bright_green",
        LatencyColour::Ansi(AnsiColors::BrightYellow) => "bright_yellow",
        LatencyColour::Ansi(AnsiColors::BrightBlue) => "bright_blue",
        LatencyColour::Ansi(AnsiColors::BrightMagenta) => "bright_magenta",
        LatencyColour::Ansi(AnsiColors::BrightCyan) => "bright_cyan",
        LatencyColour::Ansi(AnsiColors::BrightWhite) => "bright_white",
        LatencyColour::Ansi(AnsiColors::White | _) => "white",
    }
}

use crate::contract::{GenerationSpec, OutputSpec, VariantResult};
use sha2::{Digest, Sha256};

#[derive(Debug, PartialEq, Eq)]
pub enum EngineError {
    InvalidSpec(&'static str),
    InvalidSvg(&'static str),
}

impl EngineError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidSpec(_) => "invalid_spec",
            Self::InvalidSvg(_) => "invalid_svg",
        }
    }
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSpec(message) | Self::InvalidSvg(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub fn validate_result(result: &VariantResult) -> Result<(), EngineError> {
    let invalid = EngineError::InvalidSpec;
    result
        .seed
        .parse::<u64>()
        .map_err(|_| invalid("result seed exceeds u64"))?;
    result
        .index
        .parse::<u64>()
        .map_err(|_| invalid("result index exceeds u64"))?;
    if !is_hash(&result.template_hash)
        || !is_hash(&result.configuration_hash)
        || result
            .artifact_hash
            .as_ref()
            .is_some_and(|hash| !is_hash(hash))
    {
        return Err(invalid("result hashes must be lowercase SHA-256 hex"));
    }
    if !(1..=256).contains(&result.device.chars().count())
        || result
            .reason
            .as_ref()
            .is_some_and(|reason| !(1..=256).contains(&reason.chars().count()))
    {
        return Err(invalid(
            "result device and reason must contain 1 to 256 characters",
        ));
    }
    Ok(())
}

fn is_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

pub fn validate_spec(spec: &GenerationSpec) -> Result<(), EngineError> {
    let invalid = EngineError::InvalidSpec;
    spec.seed
        .parse::<u64>()
        .map_err(|_| invalid("seed exceeds u64"))?;
    let start = spec
        .start_index
        .parse::<u64>()
        .map_err(|_| invalid("startIndex exceeds u64"))?;
    let count = spec.count.get();
    if count > 10_000 || start.checked_add(count - 1).is_none() {
        return Err(invalid("variant interval exceeds supported limits"));
    }
    validate_output(&spec.output)?;
    if !is_hash(&spec.template_hash) {
        return Err(invalid("templateHash must be lowercase SHA-256 hex"));
    }
    let rules = &spec.rules;
    for range in [
        &rules.translate_x,
        &rules.translate_y,
        &rules.rotation_degrees,
        &rules.scale_x,
        &rules.scale_y,
        &rules.opacity,
    ] {
        if !range.min.is_finite() || !range.max.is_finite() || range.min > range.max {
            return Err(invalid("rule ranges must be finite and ordered"));
        }
    }
    if rules.scale_x.min <= 0.0 || rules.scale_y.min <= 0.0 {
        return Err(invalid("scale must be positive"));
    }
    if rules.opacity.min < 0.0 || rules.opacity.max > 1.0 {
        return Err(invalid("opacity must be between zero and one"));
    }
    if !rules.distortion.is_finite() || rules.distortion < 0.0 {
        return Err(invalid("distortion must be finite and nonnegative"));
    }
    if rules.palette.len() > 256 || rules.palette.iter().any(|color| !is_rgb(color)) {
        return Err(invalid(
            "palette exceeds limits or contains invalid RGB hex",
        ));
    }
    if let Some(crop) = &rules.crop
        && (![crop.x, crop.y, crop.width, crop.height]
            .iter()
            .all(|n| n.is_finite())
            || crop.x < 0.0
            || crop.y < 0.0
            || crop.width <= 0.0
            || crop.height <= 0.0
            || !(crop.x + crop.width).is_finite()
            || !(crop.y + crop.height).is_finite())
    {
        return Err(invalid("crop must be a finite positive rectangle"));
    }
    Ok(())
}

pub fn validate_output(output: &OutputSpec) -> Result<(), EngineError> {
    let invalid = EngineError::InvalidSpec;
    let width = output.width.get();
    let height = output.height.get();
    if width > 4096
        || height > 4096
        || width
            .checked_mul(height)
            .is_none_or(|pixels| pixels > 16_777_216)
    {
        return Err(invalid("output dimensions exceed supported limits"));
    }
    let background = output.background.as_str();
    if background != "transparent" && !is_rgb(background) {
        return Err(invalid("background must be transparent or RGB hex"));
    }
    Ok(())
}

fn is_rgb(color: &str) -> bool {
    color.len() == 7
        && color.starts_with('#')
        && color.bytes().skip(1).all(|c| c.is_ascii_hexdigit())
}

pub fn configuration_hash(spec: &GenerationSpec) -> Result<String, EngineError> {
    validate_spec(spec)?;
    let mut value = serde_json::to_value(spec)
        .map_err(|_| EngineError::InvalidSpec("configuration cannot be serialized"))?;
    let fields = value
        .as_object_mut()
        .ok_or(EngineError::InvalidSpec("configuration is not an object"))?;
    fields.remove("count");
    fields.remove("startIndex");
    let bytes = serde_json::to_vec(&value)
        .map_err(|_| EngineError::InvalidSpec("configuration cannot be serialized"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

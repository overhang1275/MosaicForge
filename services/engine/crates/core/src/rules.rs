use crate::{
    EngineError,
    contract::{GenerationSpec, generation::Range},
    rng::{map_u32_to_range, sample_u32},
    validate_spec,
};

const TRANSLATE_X_RULE: u16 = 1;
const TRANSLATE_Y_RULE: u16 = 2;
const ROTATION_RULE: u16 = 3;
const SCALE_X_RULE: u16 = 4;
const SCALE_Y_RULE: u16 = 5;

#[derive(Debug, PartialEq)]
pub struct TransformParameters {
    pub translate_x: f64,
    pub translate_y: f64,
    pub rotation_degrees: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub mirror_x: bool,
    pub mirror_y: bool,
}

pub fn sample_transform(
    spec: &GenerationSpec,
    index: u64,
    attempt: u8,
) -> Result<TransformParameters, EngineError> {
    validate_spec(spec)?;
    let seed = spec
        .seed
        .parse::<u64>()
        .map_err(|_| EngineError::InvalidSpec("seed exceeds u64"))?;
    let sample = |rule, range: &Range| {
        map_u32_to_range(
            sample_u32(seed, index, rule, attempt, 0)?,
            range.min,
            range.max,
        )
    };
    let scale_x = sample(SCALE_X_RULE, &spec.rules.scale_x)?;
    let scale_y = if spec.rules.preserve_aspect_ratio == Some(true) {
        scale_x
    } else {
        sample(SCALE_Y_RULE, &spec.rules.scale_y)?
    };
    Ok(TransformParameters {
        translate_x: sample(TRANSLATE_X_RULE, &spec.rules.translate_x)?,
        translate_y: sample(TRANSLATE_Y_RULE, &spec.rules.translate_y)?,
        rotation_degrees: sample(ROTATION_RULE, &spec.rules.rotation_degrees)?,
        scale_x,
        scale_y,
        mirror_x: spec.rules.mirror_x,
        mirror_y: spec.rules.mirror_y,
    })
}

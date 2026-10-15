use crate::contract::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use mosaicforge_core::{
    EngineError, contract::generation::OutputSpecFormat, validate_result, validate_spec,
};
use sha2::{Digest, Sha256};

pub const MAX_TEMPLATE_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_SPEC_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Preview,
    Export,
}

fn require(condition: bool) -> Result<(), EngineError> {
    condition
        .then_some(())
        .ok_or(EngineError::InvalidSpec("invalid preview message"))
}

fn template(text: &str) -> Result<(), EngineError> {
    require(!text.is_empty() && text.len() <= MAX_TEMPLATE_BYTES)
}

pub fn validate_inspect_request(request: &InspectRequest) -> Result<(), EngineError> {
    template(&request.template_svg)
}

pub fn validate_inspect_response(response: &InspectResponse) -> Result<(), EngineError> {
    let v = serde_json::to_value(response)
        .map_err(|_| EngineError::InvalidSpec("invalid preview metadata"))?;
    require(
        ["width", "height"]
            .iter()
            .all(|key| v[key].as_f64().is_some_and(|n| n.is_finite() && n > 0.0)),
    )?;
    let view = v["viewBox"]
        .as_array()
        .ok_or(EngineError::InvalidSpec("invalid viewBox"))?;
    require(
        view.len() == 4
            && view.iter().all(|n| n.as_f64().is_some_and(f64::is_finite))
            && view[2].as_f64().is_some_and(|n| n > 0.0)
            && view[3].as_f64().is_some_and(|n| n > 0.0),
    )?;
    require(
        v["layerIds"].as_array().is_some_and(|a| a.len() <= 20_000)
            && v["segmentCount"].as_u64().is_some_and(|n| n <= 100_000),
    )
}

pub fn validate_generation_request(
    request: &GenerationRequest,
    operation: Operation,
) -> Result<(), EngineError> {
    template(&request.template_svg)?;
    let spec = &request.generation;
    validate_spec(spec)?;
    require(
        spec.count.get() == 1 && spec.output.width.get() <= 512 && spec.output.height.get() <= 512,
    )?;
    require(operation != Operation::Preview || spec.output.format == OutputSpecFormat::Png)?;
    let bytes = serde_json::to_vec(spec)
        .map_err(|_| EngineError::InvalidSpec("invalid preview configuration"))?;
    require(bytes.len() <= MAX_SPEC_BYTES)
}

pub fn validate_generation_response(
    response: &GenerationResponse,
    operation: Operation,
) -> Result<(), EngineError> {
    validate_result(&response.variant)?;
    let value = serde_json::to_value(response)
        .map_err(|_| EngineError::InvalidSpec("invalid preview response"))?;
    let variant = &value["variant"];
    if variant["status"] == "rejected" {
        return require(
            response.artifact.is_none()
                && response.variant.artifact_hash.is_none()
                && response.variant.reason.is_some(),
        );
    }
    require(response.variant.reason.is_none())?;
    let artifact = value.get("artifact").ok_or(EngineError::InvalidSpec(
        "accepted response requires artifact",
    ))?;
    require(operation != Operation::Preview || artifact["mimeType"] == "image/png")?;
    let encoded = artifact["base64"]
        .as_str()
        .ok_or(EngineError::InvalidSpec("invalid artifact encoding"))?;
    require(!encoded.is_empty() && encoded.len() <= 16 * 1024 * 1024)?;
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| EngineError::InvalidSpec("invalid artifact encoding"))?;
    require(STANDARD.encode(&bytes) == encoded)?;
    require(
        variant["artifactHash"]
            .as_str()
            .is_some_and(|hash| hash == format!("{:x}", Sha256::digest(&bytes))),
    )
}

pub fn validate_error(error: &PreviewError) -> Result<(), EngineError> {
    require((1..=256).contains(&error.message.chars().count()))
}

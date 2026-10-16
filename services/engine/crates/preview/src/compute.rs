use crate::{contract::*, validation::*};
use base64::{Engine, engine::general_purpose::STANDARD};
use mosaicforge_core::{EngineError, template::import_svg};
use serde_json::{Value, json};

fn compute(path: &str, bytes: &[u8]) -> Result<Value, (&'static str, u16, String)> {
    let input: Value = serde_json::from_slice(bytes)
        .map_err(|_| ("invalid_request", 400, crate::http::UNKNOWN_REQUEST.into()))?;
    let id = input
        .get("requestId")
        .and_then(Value::as_str)
        .filter(|s| {
            s.len() == 36
                && s.bytes().enumerate().all(|(i, b)| {
                    if [8, 13, 18, 23].contains(&i) {
                        b == b'-'
                    } else {
                        b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
                    }
                })
        })
        .unwrap_or(crate::http::UNKNOWN_REQUEST)
        .to_owned();
    let invalid = || ("invalid_request", 400, id.clone());
    let engine_error = |e: EngineError| {
        (
            match e {
                EngineError::InvalidSvg(_) => "invalid_template",
                EngineError::InvalidSpec(_) => "invalid_request",
                _ => "internal_error",
            },
            if matches!(e, EngineError::RenderFailed(_)) {
                500
            } else {
                400
            },
            id.clone(),
        )
    };
    if path == "/v1/inspect" {
        let request: InspectRequest = serde_json::from_value(input).map_err(|_| invalid())?;
        validate_inspect_request(&request).map_err(|_| invalid())?;
        let t = import_svg(request.template_svg.as_bytes()).map_err(engine_error)?;
        let value = json!({"requestId":id,"templateHash":t.hash,"width":t.width,"height":t.height,"viewBox":t.view_box,"layerIds":t.layer_ids,"segmentCount":t.segment_count,"engineVersion":"0.1.0","rulesVersion":"1"});
        let response: InspectResponse = serde_json::from_value(value.clone())
            .map_err(|_| ("internal_error", 500, id.clone()))?;
        validate_inspect_response(&response).map_err(|_| ("internal_error", 500, id))?;
        return Ok(value);
    }
    let op = if path == "/v1/preview" {
        Operation::Preview
    } else {
        Operation::Export
    };
    let request: GenerationRequest = serde_json::from_value(input).map_err(|_| invalid())?;
    validate_generation_request(&request, op).map_err(|_| invalid())?;
    let template = import_svg(request.template_svg.as_bytes()).map_err(engine_error)?;
    let index = request
        .generation
        .start_index
        .parse::<u64>()
        .map_err(|_| invalid())?;
    let output = mosaicforge_cpu::export::generate_and_export(
        &template,
        &request.generation,
        index,
        mosaicforge_cpu::render::MAX_RENDER_BUFFER_BYTES,
    )
    .map_err(engine_error)?;
    let artifact=output.artifact.map(|bytes|json!({"mimeType":if request.generation.output.format==mosaicforge_core::contract::generation::OutputSpecFormat::Png{"image/png"}else{"image/svg+xml"},"base64":STANDARD.encode(bytes)}));
    let value = json!({"requestId":id,"variant":output.variant.result,"artifact":artifact});
    let response: GenerationResponse =
        serde_json::from_value(value.clone()).map_err(|_| ("internal_error", 500, id.clone()))?;
    validate_generation_response(&response, op).map_err(|_| ("internal_error", 500, id))?;
    Ok(value)
}
pub fn execute(path: &str, bytes: &[u8]) -> (u16, Vec<u8>) {
    let (status, value) = match compute(path, bytes) {
        Ok(value) => (200, value),
        Err((code, status, id)) => (
            status,
            json!({"requestId":id,"code":code,"message":"Preview operation could not be completed"}),
        ),
    };
    let body = serde_json::to_vec(&value).expect("finite validated response");
    if body.len() > crate::http::MAX_RESPONSE_BYTES {
        return (500,json!({"requestId":value["requestId"],"code":"internal_error","message":"Response exceeds limit"}).to_string().into_bytes());
    }
    (status, body)
}

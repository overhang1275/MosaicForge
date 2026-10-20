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

pub fn validate_pdf_export_request(request: &PdfExportRequest) -> Result<(), EngineError> {
    template(&request.template_svg)?;
    mosaicforge_core::pdf::validate_pdf_job(&request.job)?;
    let g = &request.job.generation;
    require(g.count.get() <= 12 && g.output.width.get() <= 512 && g.output.height.get() <= 512)?;
    require(
        serde_json::to_vec(&request.job)
            .map_err(|_| EngineError::InvalidSpec("invalid PDF job"))?
            .len()
            <= MAX_SPEC_BYTES,
    )
}
pub fn validate_selected_pdf_export_request(
    request: &SelectedPdfExportRequest,
) -> Result<Vec<u64>, EngineError> {
    let legacy: PdfExportRequest = serde_json::from_value(serde_json::json!({"requestId":request.request_id,"templateSvg":request.template_svg,"job":request.job}))
        .map_err(|_| EngineError::InvalidSpec("invalid PDF selection"))?;
    validate_pdf_export_request(&legacy)?;
    let indices = request
        .variant_indices
        .iter()
        .map(|index| {
            index
                .parse::<u64>()
                .map_err(|_| EngineError::InvalidSpec("invalid PDF index"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    mosaicforge_core::pdf::validate_pdf_selection(&request.job, &indices)?;
    Ok(indices)
}

pub fn validate_selected_pdf_export_response(
    response: &SelectedPdfExportResponse,
) -> Result<(), EngineError> {
    let legacy: PdfExportResponse = serde_json::from_value(serde_json::json!({"requestId":response.request_id,"document":response.document,"artifact":response.artifact}))
        .map_err(|_| EngineError::InvalidSpec("invalid selected PDF result"))?;
    validate_pdf_export_response(&legacy)?;
    let indices = response
        .variant_indices
        .iter()
        .map(|index| {
            index
                .parse::<u64>()
                .map_err(|_| EngineError::InvalidSpec("invalid PDF index"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    require(
        indices.len() as i64 == response.document.count
            && indices.first().map(u64::to_string).as_deref()
                == Some(response.document.start_index.as_str())
            && indices.windows(2).all(|pair| pair[0] < pair[1]),
    )
}
pub fn validate_pdf_download_response(response: &PdfDownloadResponse) -> Result<(), EngineError> {
    mosaicforge_core::pdf::validate_pdf_document_result(&response.document)?;
    let document = &response.document;
    let indices = response
        .variant_indices
        .iter()
        .map(|index| {
            index
                .parse::<u64>()
                .map_err(|_| EngineError::InvalidSpec("invalid PDF index"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    require(
        document.count <= 12
            && document.page_count == document.count
            && document.rejected_count == 0
            && document.status.to_string() == "completed"
            && document.document_hash.is_some()
            && indices.len() as i64 == document.count
            && indices.first().map(u64::to_string).as_deref()
                == Some(document.start_index.as_str())
            && indices.windows(2).all(|pair| pair[0] < pair[1]),
    )
}
pub fn validate_pdf_export_response(response: &PdfExportResponse) -> Result<(), EngineError> {
    mosaicforge_core::pdf::validate_pdf_document_result(&response.document)?;
    let r = &response.document;
    require(
        r.count <= 12
            && r.page_count == r.count
            && r.rejected_count == 0
            && r.status.to_string() == "completed",
    )?;
    let v = serde_json::to_value(&response.artifact)
        .map_err(|_| EngineError::InvalidSpec("invalid PDF artifact"))?;
    let encoded = v["base64"]
        .as_str()
        .ok_or(EngineError::InvalidSpec("invalid PDF encoding"))?;
    require(encoded.len() <= 16 * 1024 * 1024)?;
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| EngineError::InvalidSpec("invalid PDF encoding"))?;
    require(
        STANDARD.encode(&bytes) == encoded
            && bytes.starts_with(b"%PDF-1.7")
            && r.document_hash
                .as_ref()
                .is_some_and(|h| h.to_string() == format!("{:x}", Sha256::digest(&bytes))),
    )
}

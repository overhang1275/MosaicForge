use crate::{contract::*, validation::*};
use base64::{Engine, engine::general_purpose::STANDARD};
use mosaicforge_core::{EngineError, template::import_svg};
use serde_json::{Value, json};

fn compute(
    path: &str,
    bytes: &[u8],
    pdf_limit: usize,
) -> Result<Value, (&'static str, u16, String)> {
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
    if path == "/v1/export-pdf" {
        let request: PdfExportRequest = serde_json::from_value(input).map_err(|_| invalid())?;
        validate_pdf_export_request(&request).map_err(|_| invalid())?;
        let template = import_svg(request.template_svg.as_bytes()).map_err(engine_error)?;
        let mut output = PdfBuffer::new(pdf_limit);
        let document =
            mosaicforge_cpu::pdf::generate_pdf(&template, &request.job, 1, &mut output, |record| {
                if record.variant.status.to_string() == "rejected" {
                    return Err(EngineError::InvalidSpec(
                        "PDF series contains rejected variants",
                    ));
                }
                Ok(())
            });
        if output.exceeded {
            return Err(("payload_too_large", 413, id));
        }
        let document = document.map_err(engine_error)?;
        let value = json!({"requestId":id,"document":document,"artifact":{"mimeType":"application/pdf","base64":STANDARD.encode(output.bytes)}});
        let response: PdfExportResponse = serde_json::from_value(value.clone())
            .map_err(|_| ("internal_error", 500, id.clone()))?;
        validate_pdf_export_response(&response).map_err(|_| ("internal_error", 500, id))?;
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
    let (status, value) = match compute(path, bytes, MAX_PDF_BYTES) {
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

// Leave space for JSON metadata and base64 expansion inside the 16 MiB response cap.
const MAX_PDF_BYTES: usize = 12 * 1024 * 1024 - 64 * 1024;
struct PdfBuffer {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}
impl PdfBuffer {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            exceeded: false,
        }
    }
}
impl std::io::Write for PdfBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(std::io::Error::other(
                "PDF response exceeds local preview limit",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn oversized_pdf_returns_payload_error_without_document() {
        let input = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../contracts/preview/v1/fixtures/export-pdf.json"
        ));
        let error = compute("/v1/export-pdf", input, 16).unwrap_err();
        assert_eq!((error.0, error.1), ("payload_too_large", 413));
        assert_eq!(error.2, "12345678-1234-1234-1234-123456789abc");
    }
    #[test]
    fn pdf_buffer_never_grows_past_limit() {
        let mut b = PdfBuffer::new(MAX_PDF_BYTES);
        assert!(b.write_all(&vec![0; 12 * 1024 * 1024]).is_err());
        assert!(b.bytes.is_empty());
    }
}

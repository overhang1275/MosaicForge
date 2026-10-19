use crate::{EngineError, contract::PdfJobSpec};
use sha2::{Digest, Sha256};
#[derive(Debug, Clone, Copy)]
pub struct PageBoxes {
    pub media: [f32; 4],
    pub trim: [f32; 4],
}
pub fn validate_pdf_job(job: &PdfJobSpec) -> Result<(), EngineError> {
    crate::validate_spec(&job.generation)?;
    let p = &job.pdf;
    if job.generation.output.format != crate::contract::generation::OutputSpecFormat::Svg
        || !p.trim_width_mm.is_finite()
        || !(1.0..=2000.0).contains(&p.trim_width_mm)
        || !p.trim_height_mm.is_finite()
        || !(1.0..=2000.0).contains(&p.trim_height_mm)
        || !p.bleed_mm.is_finite()
        || !(0.0..=50.0).contains(&p.bleed_mm)
    {
        return Err(EngineError::InvalidSpec(
            "PDF requires SVG generation and finite supported millimetre dimensions",
        ));
    }
    Ok(())
}
pub fn page_boxes(job: &PdfJobSpec) -> Result<PageBoxes, EngineError> {
    validate_pdf_job(job)?;
    let p = &job.pdf;
    let pt = |mm: f64| (mm * 72.0 / 25.4) as f32;
    Ok(PageBoxes {
        media: [
            0.0,
            0.0,
            pt(p.trim_width_mm + 2.0 * p.bleed_mm),
            pt(p.trim_height_mm + 2.0 * p.bleed_mm),
        ],
        trim: [
            pt(p.bleed_mm),
            pt(p.bleed_mm),
            pt(p.bleed_mm + p.trim_width_mm),
            pt(p.bleed_mm + p.trim_height_mm),
        ],
    })
}
pub fn pdf_configuration_hash(job: &PdfJobSpec) -> Result<String, EngineError> {
    validate_pdf_job(job)?;
    let value = serde_json::to_value(job)
        .map_err(|_| EngineError::InvalidSpec("cannot encode PDF configuration"))?;
    let bytes = serde_json::to_vec(&value)
        .map_err(|_| EngineError::InvalidSpec("cannot encode PDF configuration"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn validate_pdf_selection(job: &PdfJobSpec, indices: &[u64]) -> Result<(), EngineError> {
    validate_pdf_job(job)?;
    if indices.len() as u64 != job.generation.count.get()
        || indices.first().map(u64::to_string).as_deref()
            != Some(job.generation.start_index.as_str())
        || indices.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(EngineError::InvalidSpec(
            "PDF selection must be ordered, unique and match count/startIndex",
        ));
    }
    Ok(())
}

pub fn pdf_selection_configuration_hash(
    job: &PdfJobSpec,
    indices: &[u64],
) -> Result<String, EngineError> {
    validate_pdf_selection(job, indices)?;
    let value = serde_json::json!({"schemaVersion":"2", "job":job, "variantIndices":indices.iter().map(u64::to_string).collect::<Vec<_>>()});
    let bytes = serde_json::to_vec(&value)
        .map_err(|_| EngineError::InvalidSpec("cannot encode PDF selection"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn validate_pdf_document_result(
    r: &crate::contract::PdfDocumentResult,
) -> Result<(), EngineError> {
    let invalid = || EngineError::InvalidSpec("inconsistent PDF document result");
    let start = r.start_index.parse::<u64>().map_err(|_| invalid())?;
    r.seed.parse::<u64>().map_err(|_| invalid())?;
    if !(1..=10000).contains(&r.count)
        || r.requested_count != r.count
        || !(0..=r.count).contains(&r.accepted_count)
        || !(0..=r.count).contains(&r.rejected_count)
        || r.accepted_count + r.rejected_count != r.count
        || r.page_count != r.accepted_count
        || start.checked_add(r.count as u64 - 1).is_none()
    {
        return Err(invalid());
    }
    let completed = r.status.to_string() == "completed";
    if completed != (r.page_count > 0) || completed != r.document_hash.is_some() {
        return Err(invalid());
    }
    Ok(())
}

pub fn validate_pdf_page_record(r: &crate::contract::PdfPageRecord) -> Result<(), EngineError> {
    crate::validate_result(&r.variant)?;
    let accepted = r.variant.status.to_string() == "accepted";
    if accepted != r.page_number.is_some()
        || accepted != r.variant.artifact_hash.is_some()
        || accepted == r.variant.reason.is_some()
        || r.page_number.is_some_and(|p| p.get() > 10000)
    {
        return Err(EngineError::InvalidSpec("inconsistent PDF page record"));
    }
    Ok(())
}

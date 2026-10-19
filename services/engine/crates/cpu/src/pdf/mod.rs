pub mod geometry;
mod writer;

use mosaicforge_core::{
    EngineError,
    contract::{PdfDocumentResult, PdfJobSpec, PdfPageRecord},
    template::Template,
};
use std::{
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};
/// Writes one complete PDF to a caller-owned temporary destination. Errors leave it unpublished.
/// Records are emitted incrementally; callbacks must not treat them as a completed document.
pub fn generate_pdf<W: Write, F: FnMut(PdfPageRecord) -> Result<(), EngineError>>(
    template: &Template,
    job: &PdfJobSpec,
    batch_size: u64,
    output: &mut W,
    records: F,
) -> Result<PdfDocumentResult, EngineError> {
    generate_pdf_cancellable(
        template,
        job,
        batch_size,
        output,
        records,
        &AtomicBool::new(false),
    )?
    .ok_or(EngineError::RenderFailed("unexpected PDF cancellation"))
}

/// None means cooperative cancellation; the caller must discard its temporary.
/// Page records are provisional until a completed document is published.
pub fn generate_pdf_cancellable<W: Write, F: FnMut(PdfPageRecord) -> Result<(), EngineError>>(
    template: &Template,
    job: &PdfJobSpec,
    batch_size: u64,
    output: &mut W,
    records: F,
    token: &AtomicBool,
) -> Result<Option<PdfDocumentResult>, EngineError> {
    generate_pdf_indices(template, job, batch_size, output, records, token, None)
}

/// V2 selection: every explicit index must produce a valid page, or discard the destination.
pub fn generate_selected_pdf<W: Write, F: FnMut(PdfPageRecord) -> Result<(), EngineError>>(
    template: &Template,
    job: &PdfJobSpec,
    indices: &[u64],
    output: &mut W,
    records: F,
) -> Result<PdfDocumentResult, EngineError> {
    mosaicforge_core::pdf::validate_pdf_selection(job, indices)?;
    generate_pdf_indices(
        template,
        job,
        1,
        output,
        records,
        &AtomicBool::new(false),
        Some(indices),
    )?
    .ok_or(EngineError::RenderFailed("unexpected PDF cancellation"))
}

fn generate_pdf_indices<W: Write, F: FnMut(PdfPageRecord) -> Result<(), EngineError>>(
    template: &Template,
    job: &PdfJobSpec,
    batch_size: u64,
    output: &mut W,
    mut records: F,
    token: &AtomicBool,
    indices: Option<&[u64]>,
) -> Result<Option<PdfDocumentResult>, EngineError> {
    use mosaicforge_core::{
        configuration_hash,
        contract::pdf_result::*,
        pdf::{pdf_configuration_hash, validate_pdf_job},
    };
    validate_pdf_job(job)?;
    if batch_size == 0 {
        return Err(EngineError::InvalidSpec("PDF batch size must be positive"));
    }
    let spec = &job.generation;
    let start = spec
        .start_index
        .parse::<u64>()
        .map_err(|_| EngineError::InvalidSpec("invalid index"))?;
    let count = spec.count.get();
    let mut accepted = 0u64;
    let mut writer = writer::PdfWriter::new(output);
    let mut processed = 0;
    while processed < count {
        let group = (count - processed).min(batch_size);
        for local in 0..group {
            if token.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let index = indices.map_or(start + processed + local, |selection| {
                selection[(processed + local) as usize]
            });
            let mut selected_spec;
            let variant_spec = if indices.is_some() {
                selected_spec = spec.clone();
                selected_spec.start_index = index
                    .to_string()
                    .parse()
                    .map_err(|_| EngineError::InvalidSpec("invalid selected index"))?;
                selected_spec.count = std::num::NonZeroU64::new(1).unwrap();
                &selected_spec
            } else {
                spec
            };
            let exported = crate::export::generate_and_export(template, variant_spec, index, 0)?;
            if token.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let page_number = if let Some(svg) = exported.artifact {
                let candidate = exported
                    .variant
                    .template
                    .as_ref()
                    .ok_or(EngineError::RenderFailed("missing PDF candidate"))?;
                let viewport = mosaicforge_core::template::prepare_viewport(
                    candidate.normalized_svg.as_bytes(),
                    &spec.output,
                )?;
                let coverage =
                    geometry::measure_artwork_coverage(viewport.normalized_svg.as_bytes(), job)?;
                if coverage.rejection_reason().is_some() {
                    return Err(EngineError::RenderFailed(
                        "physical PDF cover fails artwork coverage; adjust layout or rules",
                    ));
                }
                let page = geometry::optimize_page(&svg, job)?;
                if token.load(Ordering::Relaxed) {
                    return Ok(None);
                }
                writer.page(&page)?;
                accepted += 1;
                std::num::NonZeroU64::new(accepted)
            } else {
                if indices.is_some() {
                    return Err(EngineError::InvalidSpec(
                        "selected PDF contains rejected variants",
                    ));
                }
                None
            };
            let record = PdfPageRecord {
                variant: exported.variant.result,
                page_number,
            };
            mosaicforge_core::pdf::validate_pdf_page_record(&record)?;
            records(record)?;
        }
        processed += group;
    }
    if token.load(Ordering::Relaxed) {
        return Ok(None);
    }
    let digest = writer.finish()?;
    let encode = |_| EngineError::InvalidSpec("cannot encode PDF result");
    let result = PdfDocumentResult {
        schema_version: PdfDocumentResultSchemaVersion::X1,
        exporter_version: PdfDocumentResultExporterVersion::X1,
        template_hash: spec.template_hash.as_str().parse().map_err(encode)?,
        seed: spec.seed.as_str().parse().map_err(encode)?,
        start_index: spec.start_index.as_str().parse().map_err(encode)?,
        count: count as i64,
        requested_count: count as i64,
        accepted_count: accepted as i64,
        rejected_count: (count - accepted) as i64,
        page_count: accepted as i64,
        generation_configuration_hash: configuration_hash(spec)?.parse().map_err(encode)?,
        document_configuration_hash: match indices {
            Some(selection) => {
                mosaicforge_core::pdf::pdf_selection_configuration_hash(job, selection)?
            }
            None => pdf_configuration_hash(job)?,
        }
        .parse()
        .map_err(encode)?,
        document_hash: digest.map(|h| h.parse()).transpose().map_err(encode)?,
        status: if accepted == 0 {
            PdfDocumentResultStatus::NoPages
        } else {
            PdfDocumentResultStatus::Completed
        },
    };
    mosaicforge_core::pdf::validate_pdf_document_result(&result)?;
    Ok(Some(result))
}

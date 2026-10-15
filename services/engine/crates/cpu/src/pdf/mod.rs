pub mod geometry;
mod writer;

use mosaicforge_core::{
    EngineError,
    contract::{PdfDocumentResult, PdfJobSpec, PdfPageRecord},
    template::Template,
};
use std::io::Write;
/// Writes one complete PDF to a caller-owned temporary destination. Errors leave it unpublished.
/// Records are emitted incrementally; callbacks must not treat them as a completed document.
pub fn generate_pdf<W: Write, F: FnMut(PdfPageRecord) -> Result<(), EngineError>>(
    template: &Template,
    job: &PdfJobSpec,
    batch_size: u64,
    output: &mut W,
    mut records: F,
) -> Result<PdfDocumentResult, EngineError> {
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
            let index = start + processed + local;
            let exported = crate::export::generate_and_export(template, spec, index, 0)?;
            let page_number = if let Some(svg) = exported.artifact {
                let page = geometry::optimize_page(&svg, job)?;
                writer.page(&page)?;
                accepted += 1;
                std::num::NonZeroU64::new(accepted)
            } else {
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
        document_configuration_hash: pdf_configuration_hash(job)?.parse().map_err(encode)?,
        document_hash: digest.map(|h| h.parse()).transpose().map_err(encode)?,
        status: if accepted == 0 {
            PdfDocumentResultStatus::NoPages
        } else {
            PdfDocumentResultStatus::Completed
        },
    };
    mosaicforge_core::pdf::validate_pdf_document_result(&result)?;
    Ok(result)
}

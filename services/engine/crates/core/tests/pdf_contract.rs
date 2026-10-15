use mosaicforge_core::{
    contract::PdfJobSpec,
    pdf::{page_boxes, pdf_configuration_hash, validate_pdf_job},
};
use serde_json::{Value, json};

#[test]
fn document_and_page_metadata_require_consistent_states_counts_and_hashes() {
    use mosaicforge_core::pdf::{validate_pdf_document_result, validate_pdf_page_record};
    let hash = "0".repeat(64);
    let mut result = json!({"schemaVersion":"1","exporterVersion":"1","templateHash":hash,"seed":"1","startIndex":"0","count":2,"generationConfigurationHash":hash,"documentConfigurationHash":hash,"status":"completed","requestedCount":2,"acceptedCount":2,"rejectedCount":0,"pageCount":2,"documentHash":hash});
    let parse = |v: Value| serde_json::from_value(v).unwrap();
    validate_pdf_document_result(&parse(result.clone())).unwrap();
    result.as_object_mut().unwrap().remove("documentHash");
    assert!(validate_pdf_document_result(&parse(result.clone())).is_err());
    result["status"] = json!("no_pages");
    assert!(validate_pdf_document_result(&parse(result.clone())).is_err());
    result["acceptedCount"] = json!(0);
    result["pageCount"] = json!(0);
    result["rejectedCount"] = json!(2);
    validate_pdf_document_result(&parse(result)).unwrap();
    let mut page = json!({"pageNumber":1,"variant":{"schemaVersion":"1","engineVersion":"0.1.0","rulesVersion":"1","backend":"cpu","device":"cpu","seed":"1","index":"0","templateHash":hash,"configurationHash":hash,"artifactHash":hash,"status":"accepted"}});
    let parse = |v: Value| serde_json::from_value(v).unwrap();
    validate_pdf_page_record(&parse(page.clone())).unwrap();
    page["variant"]["status"] = json!("rejected");
    assert!(validate_pdf_page_record(&parse(page)).is_err());
}
fn job() -> PdfJobSpec {
    let generation: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    serde_json::from_value(json!({"schemaVersion":"1","generation":generation,"pdf":{"exporterVersion":"1","trimWidthMm":100,"trimHeightMm":100,"bleedMm":0}})).unwrap()
}
#[test]
fn physical_boxes_are_explicit_and_independent_of_dpi() {
    let mut j = job();
    validate_pdf_job(&j).unwrap();
    let a = page_boxes(&j).unwrap();
    assert!((a.media[2] - 283.46457).abs() < 0.0001);
    assert_eq!(a.media, a.trim);
    j.pdf.bleed_mm = 3.0;
    let b = page_boxes(&j).unwrap();
    assert!((b.trim[0] - 8.503937).abs() < 0.0001);
    assert!((b.media[2] - 300.47244).abs() < 0.0001);
    j.generation.output.width = 128.try_into().unwrap();
    assert_eq!(page_boxes(&j).unwrap().media, b.media);
}
#[test]
fn pdf_validation_and_configuration_hash_bind_all_document_inputs() {
    let mut j = job();
    let original = pdf_configuration_hash(&j).unwrap();
    j.generation.count = 1.try_into().unwrap();
    assert_ne!(pdf_configuration_hash(&j).unwrap(), original);
    j.pdf.trim_width_mm = f64::NAN;
    assert!(validate_pdf_job(&j).is_err());
    j.pdf.trim_width_mm = 100.0;
    j.pdf.bleed_mm = -1.0;
    assert!(validate_pdf_job(&j).is_err());
    j.pdf.bleed_mm = 0.0;
    j.generation.output.format = "png".parse().unwrap();
    assert!(validate_pdf_job(&j).is_err());
}

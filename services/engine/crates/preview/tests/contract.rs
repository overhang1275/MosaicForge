use mosaicforge_preview::{contract::*, validation::*};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[path = "../schema_bundle.rs"]
mod schema_bundle;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../contracts/preview/v1/fixtures")
}

fn valid(value: Value, schema: &str, op: Operation) -> bool {
    match schema {
        "inspect-request" => serde_json::from_value::<InspectRequest>(value)
            .is_ok_and(|v| validate_inspect_request(&v).is_ok()),
        "inspect-response" => serde_json::from_value::<InspectResponse>(value)
            .is_ok_and(|v| validate_inspect_response(&v).is_ok()),
        "generation-request" => serde_json::from_value::<GenerationRequest>(value)
            .is_ok_and(|v| validate_generation_request(&v, op).is_ok()),
        "generation-response" => serde_json::from_value::<GenerationResponse>(value)
            .is_ok_and(|v| validate_generation_response(&v, op).is_ok()),
        "pdf-export-request" => serde_json::from_value::<PdfExportRequest>(value)
            .is_ok_and(|v| validate_pdf_export_request(&v).is_ok()),
        "pdf-export-response" => serde_json::from_value::<PdfExportResponse>(value)
            .is_ok_and(|v| validate_pdf_export_response(&v).is_ok()),
        "error" => {
            serde_json::from_value::<PreviewError>(value).is_ok_and(|v| validate_error(&v).is_ok())
        }
        _ => panic!("unknown fixture schema"),
    }
}

#[test]
fn shared_fixture_verdicts() {
    let cases: Vec<Value> =
        serde_json::from_slice(&fs::read(fixtures().join("index.json")).unwrap()).unwrap();
    assert!(cases.len() >= 50);
    for case in cases {
        let value = serde_json::from_slice(
            &fs::read(fixtures().join(case["file"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        let op = if case["operation"] == "export" {
            Operation::Export
        } else {
            Operation::Preview
        };
        assert_eq!(
            valid(value, case["schema"].as_str().unwrap(), op),
            case["valid"].as_bool().unwrap(),
            "{}",
            case["file"]
        );
    }
}

#[test]
fn u64_roundtrip_and_existing_model_reuse() {
    let value: Value =
        serde_json::from_slice(&fs::read(fixtures().join("request-max-u64.json")).unwrap())
            .unwrap();
    let request: GenerationRequest = serde_json::from_value(value).unwrap();
    let _: &mosaicforge_core::contract::GenerationSpec = &request.generation;
    validate_generation_request(&request, Operation::Preview).unwrap();
    let output = serde_json::to_value(request).unwrap();
    assert_eq!(output["generation"]["seed"], "18446744073709551615");
    assert_eq!(output["generation"]["startIndex"], "18446744073709551615");
}

#[test]
fn unicode_limit_counts_utf8_bytes_at_boundary() {
    let mut value: Value =
        serde_json::from_slice(&fs::read(fixtures().join("inspect.json")).unwrap()).unwrap();
    value["templateSvg"] = json!("é".repeat(5 * 1024 * 1024 / 2));
    assert!(valid(value.clone(), "inspect-request", Operation::Preview));
    value["templateSvg"].as_str().unwrap();
    value["templateSvg"] = json!(format!("{}é", value["templateSvg"].as_str().unwrap()));
    assert!(!valid(value, "inspect-request", Operation::Preview));
}

#[test]
fn metadata_constructed_with_nonfinite_dimensions_is_invalid() {
    let mut meta: InspectResponse =
        serde_json::from_slice(&fs::read(fixtures().join("metadata.json")).unwrap()).unwrap();
    meta.width = f64::NAN;
    assert!(validate_inspect_response(&meta).is_err());
}

#[test]
fn build_rejects_reference_outside_contracts() {
    let root = fixtures()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    assert!(
        schema_bundle::bundle_schema(&root, &root.join("preview/v1/inspect-request.schema.json"))
            .is_ok()
    );
    for reference in [
        "../../../../AGENTS.md",
        "https://example.com/schema.json",
        "file:///etc/passwd",
    ] {
        assert!(
            schema_bundle::check_reference(
                &root,
                &root.join("preview/v1/inspect-request.schema.json"),
                reference
            )
            .is_err(),
            "{reference}"
        );
    }
}

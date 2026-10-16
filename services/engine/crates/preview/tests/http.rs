use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use mosaicforge_preview::http::{MAX_BODY_BYTES, router};
use serde_json::{Value, json};
use tower::ServiceExt;

const TOKEN: &str = "test-only-token-012345678901234567890123";
fn fixture(name: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/../../../../contracts/preview/v1/fixtures/{name}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap()
}
async fn call(path: &str, value: Value) -> (StatusCode, Value) {
    let response = router(TOKEN.into(), "127.0.0.1:4771".into())
        .oneshot(
            Request::post(path)
                .header("host", "127.0.0.1:4771")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&value).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 17 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn inspect_uses_importer_and_returns_correlation() {
    let (status, value) = call("/v1/inspect", fixture("inspect")).await;
    assert_eq!(status, StatusCode::OK);
    let expected = fixture("metadata");
    for key in [
        "requestId",
        "templateHash",
        "layerIds",
        "segmentCount",
        "engineVersion",
        "rulesVersion",
    ] {
        assert_eq!(value[key], expected[key]);
    }
    assert_eq!(value["width"].as_f64(), Some(100.0));
    assert_eq!(value["height"].as_f64(), Some(80.0));
    assert_eq!(
        value["viewBox"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_f64().unwrap())
            .collect::<Vec<_>>(),
        vec![0.0, 0.0, 100.0, 80.0]
    );
}

#[tokio::test]
async fn invalid_template_hash_contract_and_hostile_svg_are_sanitized() {
    let mut req = fixture("inspect");
    req["templateSvg"] = json!("<svg><script>SECRET-SOURCE</script></svg>");
    let (status, value) = call("/v1/inspect", req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(value["code"], "invalid_template");
    assert!(!value.to_string().contains("SECRET-SOURCE"));
    let mut req = fixture("export-svg");
    req["generation"]["templateHash"] = json!("0".repeat(64));
    assert_eq!(call("/v1/export", req).await.0, StatusCode::BAD_REQUEST);
    let mut req = fixture("export-svg");
    req["generation"]["count"] = json!(2);
    assert_eq!(call("/v1/export", req).await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn png_svg_http_bytes_match_reference_engine_and_rejection() {
    for format in ["svg", "png"] {
        let mut req = fixture("export-svg");
        req["generation"]["output"]["format"] = json!(format);
        req["templateSvg"] = json!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"80\"><rect width=\"100\" height=\"80\" fill=\"#123456\"/></svg>"
        );
        let template =
            mosaicforge_core::template::import_svg(req["templateSvg"].as_str().unwrap().as_bytes())
                .unwrap();
        req["generation"]["templateHash"] = json!(template.hash);
        let typed: mosaicforge_preview::contract::GenerationRequest =
            serde_json::from_value(req.clone()).unwrap();
        let template =
            mosaicforge_core::template::import_svg(typed.template_svg.as_bytes()).unwrap();
        let expected = mosaicforge_cpu::export::generate_and_export(
            &template,
            &typed.generation,
            u64::MAX,
            512 * 1024 * 1024,
        )
        .unwrap();
        let (status, value) = call(
            if format == "png" {
                "/v1/preview"
            } else {
                "/v1/export"
            },
            req,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            value["variant"],
            serde_json::to_value(expected.variant.result).unwrap()
        );
        assert_eq!(
            STANDARD
                .decode(value["artifact"]["base64"].as_str().unwrap())
                .unwrap(),
            expected.artifact.unwrap()
        );
    }
    let mut req = fixture("export-svg");
    req["templateSvg"] = json!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"><rect width=\"1\" height=\"1\"/></svg>"
    );
    let template =
        mosaicforge_core::template::import_svg(req["templateSvg"].as_str().unwrap().as_bytes())
            .unwrap();
    req["generation"]["templateHash"] = json!(template.hash);
    let (status, value) = call("/v1/export", req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["variant"]["status"], "rejected");
    assert!(value["artifact"].is_null());
}

#[tokio::test]
async fn authenticates_before_body_and_rejects_foreign_hosts_origins_and_media() {
    for (host, auth, origin, media, want) in [
        ("127.0.0.1:4771", "wrong", None, "application/json", 401),
        ("evil.example", TOKEN, None, "application/json", 403),
        (
            "127.0.0.1:4771",
            TOKEN,
            Some("https://evil.example"),
            "application/json",
            403,
        ),
        ("127.0.0.1:4771", TOKEN, None, "text/plain", 415),
    ] {
        let mut req = Request::post("/v1/inspect")
            .header("host", host)
            .header("authorization", format!("Bearer {auth}"))
            .header("content-type", media);
        if let Some(origin) = origin {
            req = req.header("origin", origin);
        }
        let response = router(TOKEN.into(), "127.0.0.1:4771".into())
            .oneshot(req.body(Body::from("not-json")).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), want);
    }
}

#[tokio::test]
async fn caps_body_without_content_length_and_decoded_utf8_fields() {
    let body = Body::from(vec![b' '; MAX_BODY_BYTES + 1]);
    let response = router(TOKEN.into(), "127.0.0.1:4771".into())
        .oneshot(
            Request::post("/v1/inspect")
                .header("host", "127.0.0.1:4771")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let mut req = fixture("inspect");
    req["templateSvg"] = json!("é".repeat(5 * 1024 * 1024 / 2 + 1));
    assert_eq!(call("/v1/inspect", req).await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn pdf_is_one_vector_document_and_matches_cpu_exporter() {
    let mut req = fixture("export-pdf");
    let template =
        mosaicforge_core::template::import_svg(req["templateSvg"].as_str().unwrap().as_bytes())
            .unwrap();
    req["job"]["generation"]["templateHash"] = json!(template.hash);
    let job: mosaicforge_core::contract::PdfJobSpec =
        serde_json::from_value(req["job"].clone()).unwrap();
    let mut expected = Vec::new();
    mosaicforge_cpu::pdf::generate_pdf(&template, &job, 1, &mut expected, |_| Ok(())).unwrap();
    let (status, value) = call("/v1/export-pdf", req.clone()).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["document"]["pageCount"], 3);
    assert_eq!(value["artifact"]["mimeType"], "application/pdf");
    let bytes = STANDARD
        .decode(value["artifact"]["base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(bytes, expected);
    let pdf = String::from_utf8_lossy(&bytes);
    assert!(pdf.starts_with("%PDF-1.7"));
    assert!(pdf.contains("/Count 3"));
    assert!(!pdf.contains("/Subtype /Image"));
    assert_eq!(call("/v1/export-pdf", req.clone()).await.1, value);
    for (key, bad) in [
        ("count", json!(13)),
        ("startIndex", json!("18446744073709551615")),
    ] {
        let mut invalid = req.clone();
        invalid["job"]["generation"][key] = bad;
        assert_eq!(
            call("/v1/export-pdf", invalid).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    req["job"]["generation"]["rules"]["translateX"] = json!({"min":10000,"max":10000});
    let (status, value) = call("/v1/export-pdf", req).await;
    assert_ne!(status, StatusCode::OK);
    assert!(value.get("artifact").is_none());
}

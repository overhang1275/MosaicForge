use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use serde_json::{Value, json};
use tower::ServiceExt;
fn fixture() -> Value {
    let mut v: Value = serde_json::from_str(include_str!(
        "../../../../../contracts/preview/v1/fixtures/export-pdf.json"
    ))
    .unwrap();
    v["job"]["generation"]["count"] = json!(1);
    json!({"schemaVersion":"1","jobId":"12345678-1234-1234-1234-123456789abc","blockId":"22345678-1234-1234-1234-123456789abc","templateSvg":v["templateSvg"],"job":v["job"],"startIndex":"0","candidateCount":3,"acceptedLimit":3})
}
#[test]
fn finalization_rejects_a_committed_suffix_that_skips_the_original_interval_start() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    let dir = std::env::temp_dir().join(format!("mosaic-finalize-suffix-{}", uuid::Uuid::new_v4()));
    let journal = Journal::open(&dir, JournalLimits::default()).unwrap();
    let mut block = fixture();
    block["startIndex"] = json!("1");
    let response: Value = serde_json::from_slice(
        &journal
            .execute(&serde_json::to_vec(&block).unwrap())
            .unwrap(),
    )
    .unwrap();
    let request = final_request(&block, &response, 2);
    let result = journal.finalize(&serde_json::to_vec(&request).unwrap());
    assert!(
        result.is_err(),
        "a committed suffix must not replace the original candidate interval"
    );
    drop(journal);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn finalization_startup_scavenges_cancelled_interrupted_own_temporaries() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    let dir =
        std::env::temp_dir().join(format!("mosaic-finalize-scavenge-{}", uuid::Uuid::new_v4()));
    let j = Journal::open(&dir, JournalLimits::default()).unwrap();
    let b = fixture();
    j.cancel(&serde_json::to_vec(&json!({"schemaVersion":"1","jobId":b["jobId"]})).unwrap())
        .unwrap();
    let root = dir.join("artifacts");
    let tmp = root.join(format!("{}.tmp", uuid::Uuid::new_v4()));
    std::fs::create_dir(&tmp).unwrap();
    let file = tmp.join("document.pdf");
    std::fs::File::create(&file)
        .unwrap()
        .set_len(1024 * 1024 * 1024)
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    drop(j);
    let j = Journal::open(&dir, JournalLimits::default()).unwrap();
    assert!(
        !tmp.exists(),
        "cancelled interrupted output must release aggregate quota at startup"
    );
    assert!(j.is_cancelled(b["jobId"].as_str().unwrap()));
    drop(j);
    std::fs::remove_dir_all(dir).unwrap();
}
async fn post(app: axum::Router, path: &str, v: &Value) -> (u16, Value) {
    let r = app
        .oneshot(
            Request::post(path)
                .header("host", "worker")
                .header("authorization", "Bearer test-token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(v).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status().as_u16();
    let b = to_bytes(r.into_body(), 1048576).await.unwrap();
    (status, serde_json::from_slice(&b).unwrap_or(json!(null)))
}
fn final_request(b: &Value, r: &Value, count: u64) -> Value {
    json!({"schemaVersion":"1","jobId":b["jobId"],"artifactId":uuid::Uuid::new_v4().to_string(),"templateSvg":b["templateSvg"],"job":b["job"],"requestedAcceptedCount":count,"blocks":[{"blockId":b["blockId"],"requestHash":r["requestHash"],"recordsSha256":r["recordsSha256"]}]})
}
async fn download(
    app: axum::Router,
    id: &str,
    kind: &str,
    method: &str,
    headers: &[(&str, &str)],
) -> axum::response::Response {
    let mut q = Request::builder()
        .method(method)
        .uri(format!("/internal/v1/artifacts/{id}/{kind}"))
        .header("host", "worker")
        .header("authorization", "Bearer test-token");
    for (k, v) in headers {
        q = q.header(*k, *v);
    }
    app.oneshot(q.body(Body::empty()).unwrap()).await.unwrap()
}
#[tokio::test]
async fn verified_download_ranges_head_auth_cancellation_and_truncation() {
    let dir = std::env::temp_dir().join(format!("mosaic-download-{}", uuid::Uuid::new_v4()));
    let app =
        mosaicforge_preview::http::router_with_journal("test-token".into(), "worker".into(), &dir)
            .unwrap();
    let b = fixture();
    let (_, r) = post(app.clone(), "/internal/v1/blocks", &b).await;
    let f = final_request(&b, &r, 2);
    let (s, r) = post(app.clone(), "/internal/v1/finalize", &f).await;
    assert_eq!(s, 200, "{r}");
    let id = f["artifactId"].as_str().unwrap();
    let response = download(app.clone(), id, "pdf", "GET", &[]).await;
    assert_eq!(response.status(), 200);
    let etag = response.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(etag, format!("\"{}\"", r["sha256"].as_str().unwrap()));
    let pdf = to_bytes(response.into_body(), 1048576).await.unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(pdf.len().to_string(), r["bytes"]);
    for (range, start, end) in [
        ("bytes=0-3", 0, 4),
        ("bytes=4-", 4, pdf.len()),
        ("bytes=-7", pdf.len() - 7, pdf.len()),
    ] {
        let response = download(app.clone(), id, "pdf", "GET", &[("range", range)]).await;
        assert_eq!(response.status(), 206);
        let body = to_bytes(response.into_body(), 1048576).await.unwrap();
        assert_eq!(body, &pdf[start..end]);
    }
    for bad in [
        "bytes=999999999-",
        "bytes=4-1",
        "bytes=0-1,3-4",
        "bytes=-0",
        "items=0-1",
    ] {
        let response = download(app.clone(), id, "pdf", "GET", &[("range", bad)]).await;
        assert_eq!(response.status(), 416);
        assert_eq!(
            response.headers()["content-range"],
            format!("bytes */{}", pdf.len())
        );
    }
    let response = download(
        app.clone(),
        id,
        "pdf",
        "GET",
        &[("range", "bytes=0-1"), ("if-range", "\"different\"")],
    )
    .await;
    assert_eq!(response.status(), 200);
    let response = download(
        app.clone(),
        id,
        "pdf",
        "HEAD",
        &[("range", "bytes=0-3"), ("if-range", &etag)],
    )
    .await;
    assert_eq!(response.status(), 206);
    assert_eq!(response.headers()["content-length"], "4");
    assert!(
        to_bytes(response.into_body(), 1048576)
            .await
            .unwrap()
            .is_empty()
    );
    let response = download(app.clone(), id, "manifest", "GET", &[]).await;
    assert_eq!(response.headers()["content-type"], "application/x-ndjson");
    let manifest = to_bytes(response.into_body(), 1048576).await.unwrap();
    let lines: Vec<Value> = std::str::from_utf8(&manifest)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 4);
    for line in &lines {
        let _: mosaicforge_preview::contract::manifest_record::ManifestRecord =
            serde_json::from_value(line.clone()).unwrap();
    }
    assert_eq!(lines[1]["index"], "0");
    assert_eq!(lines[2]["index"], "1");
    assert_eq!(lines[3]["kind"], "completed");
    assert_eq!(
        download(
            app.clone(),
            id,
            "pdf",
            "GET",
            &[("origin", "https://evil.test")]
        )
        .await
        .status(),
        403
    );
    assert_eq!(
        download(app.clone(), id, "unknown", "GET", &[])
            .await
            .status(),
        404
    );
    std::fs::OpenOptions::new()
        .write(true)
        .open(dir.join("artifacts").join(id).join("document.pdf"))
        .unwrap()
        .set_len(4)
        .unwrap();
    assert_eq!(
        download(app.clone(), id, "pdf", "GET", &[]).await.status(),
        500
    );
    assert_eq!(post(app.clone(), "/internal/v1/finalize", &f).await.0, 500);
    post(
        app.clone(),
        "/internal/v1/cancel",
        &json!({"schemaVersion":"1","jobId":b["jobId"]}),
    )
    .await;
    assert_eq!(
        download(app.clone(), id, "pdf", "GET", &[]).await.status(),
        404
    );
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn partition_independence_short_corrupt_references_and_restart_temporaries() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    use sha2::{Digest, Sha256};
    let dir = std::env::temp_dir().join(format!("mosaic-partitions-{}", uuid::Uuid::new_v4()));
    let j = Journal::open(&dir, JournalLimits::default()).unwrap();
    let b = fixture();
    let r: Value =
        serde_json::from_slice(&j.execute(&serde_json::to_vec(&b).unwrap()).unwrap()).unwrap();
    let first = final_request(&b, &r, 3);
    let a: Value =
        serde_json::from_slice(&j.finalize(&serde_json::to_vec(&first).unwrap()).unwrap()).unwrap();
    let mut other = first.clone();
    other["artifactId"] = json!(uuid::Uuid::new_v4().to_string());
    other["blocks"] = json!([]);
    for index in 0..3 {
        let mut part = b.clone();
        part["blockId"] = json!(uuid::Uuid::new_v4().to_string());
        part["startIndex"] = json!(index.to_string());
        part["candidateCount"] = json!(1);
        part["acceptedLimit"] = json!(1);
        let r: Value =
            serde_json::from_slice(&j.execute(&serde_json::to_vec(&part).unwrap()).unwrap())
                .unwrap();
        other["blocks"].as_array_mut().unwrap().push(json!({"blockId":part["blockId"],"requestHash":r["requestHash"],"recordsSha256":r["recordsSha256"]}));
    }
    let c: Value =
        serde_json::from_slice(&j.finalize(&serde_json::to_vec(&other).unwrap()).unwrap()).unwrap();
    for field in ["sha256", "selectionSha256", "documentConfigurationHash"] {
        assert_eq!(a[field], c[field], "{field}");
    }
    assert_eq!(
        a["selectionSha256"],
        format!(
            "{:x}",
            Sha256::digest(b"mosaicforge-selection-v1\n0\n1\n2\n")
        )
    );
    let mut short = final_request(&b, &r, 4);
    assert_eq!(
        j.finalize(&serde_json::to_vec(&short).unwrap())
            .unwrap_err()
            .status,
        409
    );
    assert!(
        !dir.join("artifacts")
            .join(short["artifactId"].as_str().unwrap())
            .exists()
    );
    short["requestedAcceptedCount"] = json!(2);
    short["blocks"][0]["recordsSha256"] = json!("0".repeat(64));
    assert_eq!(
        j.finalize(&serde_json::to_vec(&short).unwrap())
            .unwrap_err()
            .status,
        500
    );
    let recovery = final_request(&b, &r, 2);
    let tmp = dir
        .join("artifacts")
        .join(format!("{}.tmp", recovery["artifactId"].as_str().unwrap()));
    std::fs::create_dir(&tmp).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    std::fs::write(tmp.join("document.pdf"), b"%PDF partial").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            tmp.join("document.pdf"),
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
    }
    drop(j);
    let j = Journal::open(&dir, JournalLimits::default()).unwrap();
    assert!(j.finalize(&serde_json::to_vec(&recovery).unwrap()).is_ok());
    assert!(!tmp.exists());
    let mut invalid = final_request(&b, &r, 2);
    invalid["blocks"][0]["blockId"] = json!("../escape");
    assert_eq!(
        j.finalize(&serde_json::to_vec(&invalid).unwrap())
            .unwrap_err()
            .status,
        400
    );
    drop(j);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn concurrent_cancel_stops_publication_and_remains_cancelled_after_reopen() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    use std::sync::Arc;
    let dir = std::env::temp_dir().join(format!("mosaic-cancel-race-{}", uuid::Uuid::new_v4()));
    let j = Arc::new(Journal::open(&dir, JournalLimits::default()).unwrap());
    let mut b = fixture();
    b["candidateCount"] = json!(100);
    b["acceptedLimit"] = json!(100);
    let r: Value =
        serde_json::from_slice(&j.execute(&serde_json::to_vec(&b).unwrap()).unwrap()).unwrap();
    let f = final_request(&b, &r, 100);
    let tmp = dir
        .join("artifacts")
        .join(format!("{}.tmp", f["artifactId"].as_str().unwrap()));
    let clone = j.clone();
    let bytes = serde_json::to_vec(&f).unwrap();
    let running = std::thread::spawn(move || clone.finalize(&bytes));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !tmp.join("document.pdf").exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let cancel = serde_json::to_vec(&json!({"schemaVersion":"1","jobId":b["jobId"]})).unwrap();
    assert!(j.cancel(&cancel).is_ok());
    assert!(running.join().unwrap().is_err());
    assert!(!tmp.exists());
    assert!(
        j.artifact(f["artifactId"].as_str().unwrap(), "pdf")
            .is_err()
    );
    assert_eq!(
        j.execute(&serde_json::to_vec(&b).unwrap())
            .unwrap_err()
            .status,
        409
    );
    drop(j);
    let j = Journal::open(&dir, JournalLimits::default()).unwrap();
    assert_eq!(
        j.finalize(&serde_json::to_vec(&f).unwrap())
            .unwrap_err()
            .status,
        409
    );
    drop(j);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn aggregate_quota_failure_never_publishes_or_leaves_temporary() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    let dir = std::env::temp_dir().join(format!("mosaic-artifact-quota-{}", uuid::Uuid::new_v4()));
    let j = Journal::open(&dir, JournalLimits::default()).unwrap();
    let b = fixture();
    let r: Value =
        serde_json::from_slice(&j.execute(&serde_json::to_vec(&b).unwrap()).unwrap()).unwrap();
    let f = final_request(&b, &r, 2);
    let root = dir.join("artifacts");
    std::fs::create_dir(&root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let occupied = root.join(format!("{}.tmp", uuid::Uuid::new_v4()));
    std::fs::create_dir(&occupied).unwrap();
    std::fs::File::create(occupied.join("document.pdf"))
        .unwrap()
        .set_len(1024 * 1024 * 1024)
        .unwrap();
    assert_eq!(
        j.finalize(&serde_json::to_vec(&f).unwrap())
            .unwrap_err()
            .status,
        507
    );
    assert!(!root.join(f["artifactId"].as_str().unwrap()).exists());
    assert!(
        !root
            .join(format!("{}.tmp", f["artifactId"].as_str().unwrap()))
            .exists()
    );
    drop(j);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn streaming_writer_preserves_legacy_pdf_and_rejects_fallible_short_and_physical_selection() {
    use mosaicforge_core::{EngineError, contract::PdfJobSpec, template::import_svg};
    let b = fixture();
    let job: PdfJobSpec = serde_json::from_value(b["job"].clone()).unwrap();
    let template = import_svg(b["templateSvg"].as_str().unwrap().as_bytes()).unwrap();
    let mut legacy_job = job.clone();
    legacy_job.generation.count = std::num::NonZeroU64::new(3).unwrap();
    let indices = [0, 1, 2];
    let mut legacy = Vec::new();
    mosaicforge_cpu::pdf::generate_selected_pdf(
        &template,
        &legacy_job,
        &indices,
        &mut legacy,
        |_| Ok(()),
    )
    .unwrap();
    let mut streamed = Vec::new();
    mosaicforge_cpu::pdf::write_selected_pdf_stream(
        &template,
        &job,
        3,
        indices.map(Ok),
        &mut streamed,
        || false,
        |_, _| Ok(()),
    )
    .unwrap();
    assert_eq!(legacy, streamed);
    for selection in [
        vec![Ok(0)],
        vec![Ok(0), Err(EngineError::InvalidSpec("journal unreadable"))],
        vec![Ok(1), Ok(0)],
    ] {
        assert!(
            mosaicforge_cpu::pdf::write_selected_pdf_stream(
                &template,
                &job,
                2,
                selection,
                &mut Vec::new(),
                || false,
                |_, _| Ok(())
            )
            .is_err()
        );
    }
    let mut rejected = b["job"].clone();
    rejected["generation"]["rules"]["translateX"] = json!({"min":10000,"max":10000});
    let job: PdfJobSpec = serde_json::from_value(rejected).unwrap();
    assert!(
        mosaicforge_cpu::pdf::write_selected_pdf_stream(
            &template,
            &job,
            1,
            [Ok(0)],
            &mut Vec::new(),
            || false,
            |_, _| Ok(())
        )
        .is_err()
    );
    let mut checks = 0;
    let original_job: PdfJobSpec = serde_json::from_value(b["job"].clone()).unwrap();
    assert!(
        mosaicforge_cpu::pdf::write_selected_pdf_stream(
            &template,
            &original_job,
            3,
            [Ok(0), Ok(1), Ok(2)],
            &mut Vec::new(),
            || {
                checks += 1;
                checks > 2
            },
            |_, _| Ok(())
        )
        .is_err()
    );
}
#[tokio::test]
async fn seal_exact_selection_replay_restart_cancel_and_corruption() {
    let dir = std::env::temp_dir().join(format!("mosaic-finalize-{}", uuid::Uuid::new_v4()));
    let app =
        mosaicforge_preview::http::router_with_journal("test-token".into(), "worker".into(), &dir)
            .unwrap();
    let b = fixture();
    let (s, r) = post(app.clone(), "/internal/v1/blocks", &b).await;
    assert_eq!(s, 200);
    let f = json!({"schemaVersion":"1","jobId":b["jobId"],"artifactId":"32345678-1234-1234-1234-123456789abc","templateSvg":b["templateSvg"],"job":b["job"],"requestedAcceptedCount":2,"blocks":[{"blockId":b["blockId"],"requestHash":r["requestHash"],"recordsSha256":r["recordsSha256"]}]});
    let (s, result) = post(app.clone(), "/internal/v1/finalize", &f).await;
    assert_eq!(s, 200, "{result}");
    assert_eq!(result["pageCount"], 2);
    assert_eq!(
        post(app.clone(), "/internal/v1/finalize", &f).await,
        (200, result.clone())
    );
    let mut changed = f.clone();
    changed["requestedAcceptedCount"] = json!(1);
    assert_eq!(
        post(app.clone(), "/internal/v1/finalize", &changed).await.0,
        409
    );
    drop(app);
    let app =
        mosaicforge_preview::http::router_with_journal("test-token".into(), "worker".into(), &dir)
            .unwrap();
    assert_eq!(
        post(app.clone(), "/internal/v1/finalize", &f).await,
        (200, result)
    );
    assert_eq!(
        post(
            app.clone(),
            "/internal/v1/cancel",
            &json!({"schemaVersion":"1","jobId":b["jobId"]})
        )
        .await
        .0,
        200
    );
    assert_eq!(post(app.clone(), "/internal/v1/finalize", &f).await.0, 409);
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

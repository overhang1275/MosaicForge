use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use mosaicforge_preview::http::router_with_journal;
use serde_json::{Value, json};
use tower::ServiceExt;
fn fixture() -> Value {
    let mut v: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../contracts/preview/v1/fixtures/export-pdf.json"
    )))
    .unwrap();
    v["job"]["generation"]["count"] = json!(1);
    json!({"schemaVersion":"1","jobId":"12345678-1234-1234-1234-123456789abc","blockId":"22345678-1234-1234-1234-123456789abc","templateSvg":v["templateSvg"],"job":v["job"],"startIndex":"0","candidateCount":3,"acceptedLimit":1})
}
async fn call(app: axum::Router, v: &Value) -> (u16, Vec<u8>) {
    let r = app
        .oneshot(
            Request::post("/internal/v1/blocks")
                .header("host", "127.0.0.1:4771")
                .header("authorization", "Bearer test-token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(v).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status().as_u16();
    (
        status,
        to_bytes(r.into_body(), 16 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
}
#[tokio::test]
async fn journal_restart_replay_conflict_corruption_and_exact_stop() {
    let dir = std::env::temp_dir().join(format!("mosaic-block-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let app = router_with_journal("test-token".into(), "127.0.0.1:4771".into(), &dir).unwrap();
    let v = fixture();
    let (s, bytes) = call(app.clone(), &v).await;
    assert_eq!(s, 200, "{}", String::from_utf8_lossy(&bytes));
    let r: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(r["evaluatedCount"], 1);
    assert_eq!(r["acceptedCount"], 1);
    assert_eq!(r["nextCandidateIndex"], "1");
    drop(app);
    let app = router_with_journal("test-token".into(), "127.0.0.1:4771".into(), &dir).unwrap();
    assert_eq!(call(app.clone(), &v).await, (200, bytes));
    let mut changed = v.clone();
    changed["acceptedLimit"] = json!(2);
    assert_eq!(call(app.clone(), &changed).await.0, 409);
    let mut changed_job = v.clone();
    changed_job["jobId"] = json!("32345678-1234-1234-1234-123456789abc");
    assert_eq!(call(app.clone(), &changed_job).await.0, 409);
    let file = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .unwrap();
    std::fs::write(file, b"corrupt").unwrap();
    assert_eq!(call(app, &v).await.0, 500);
    std::fs::remove_dir_all(&dir).unwrap();
}
#[tokio::test]
async fn final_u64_index_has_no_successor_and_overflow_is_rejected() {
    let dir = std::env::temp_dir().join(format!("mosaic-block-max-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let app = router_with_journal("test-token".into(), "127.0.0.1:4771".into(), &dir).unwrap();
    let mut v = fixture();
    v["startIndex"] = json!(u64::MAX.to_string());
    v["candidateCount"] = json!(1);
    let (s, b) = call(app.clone(), &v).await;
    assert_eq!(s, 200, "{}", String::from_utf8_lossy(&b));
    let r: Value = serde_json::from_slice(&b).unwrap();
    assert!(r["nextCandidateIndex"].is_null());
    v["candidateCount"] = json!(2);
    assert_eq!(call(app, &v).await.0, 400);
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn physical_pdf_rejections_are_records_and_rules3_preserves_negative_space() {
    let dir = std::env::temp_dir().join(format!("mosaic-block-physical-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let app = router_with_journal("test-token".into(), "127.0.0.1:4771".into(), &dir).unwrap();
    let mut v = fixture();
    let source = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="45" width="10" height="100"/><rect y="45" width="100" height="10"/></svg>"#;
    v["templateSvg"] = json!(source);
    v["job"]["generation"]["templateHash"] = json!(
        mosaicforge_core::template::import_svg(source.as_bytes())
            .unwrap()
            .hash
    );
    v["job"]["generation"]["rulesVersion"] = json!("3");
    v["job"]["generation"]["rules"]["preserveAspectRatio"] = json!(true);
    for (bleed, scale, accepted) in [(0, 1, 1), (3, 1, 0), (3, 4, 1)] {
        v["blockId"] = json!(uuid::Uuid::new_v4().to_string());
        v["candidateCount"] = json!(2);
        v["job"]["pdf"]["bleedMm"] = json!(bleed);
        v["job"]["generation"]["rules"]["scaleX"] = json!({"min":scale,"max":scale});
        v["job"]["generation"]["rules"]["scaleY"] = json!({"min":scale,"max":scale});
        let (s, b) = call(app.clone(), &v).await;
        assert_eq!(s, 200, "{}", String::from_utf8_lossy(&b));
        let r: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(r["acceptedCount"], accepted);
        if accepted == 0 {
            assert_eq!(r["evaluatedCount"], 2);
            assert_eq!(r["records"][0]["reason"], "physical_frame_uncovered");
        }
    }
    v["blockId"] = json!(uuid::Uuid::new_v4().to_string());
    v["job"]["generation"]["rules"]["scaleX"] = json!({"min":1,"max":1});
    v["job"]["generation"]["rules"]["scaleY"] = json!({"min":1,"max":1});
    v["job"]["generation"]["rules"]["crop"] = json!({"x":0,"y":0,"width":20,"height":20});
    v["job"]["pdf"]["bleedMm"] = json!(0);
    let (s, b) = call(app, &v).await;
    assert_eq!(s, 200);
    let r: Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(r["acceptedCount"], 0);
    assert!(r["records"][0]["reason"].is_string());
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn journal_quotas_timeout_and_request_limits_leave_no_committed_record() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    use std::time::Duration;
    let dir = std::env::temp_dir().join(format!("mosaic-block-quota-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for limits in [
        JournalLimits {
            bytes: 1,
            ..Default::default()
        },
        JournalLimits {
            files: 1,
            ..Default::default()
        },
        JournalLimits {
            time: Duration::from_nanos(1),
            ..Default::default()
        },
    ] {
        let journal = Journal::open(&dir, limits).unwrap();
        let failure = journal
            .execute(&serde_json::to_vec(&fixture()).unwrap())
            .unwrap_err();
        assert!([503, 507].contains(&failure.status));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    }
    let journal = Journal::open(&dir, JournalLimits::default()).unwrap();
    for (field, bad) in [
        ("candidateCount", json!(0)),
        ("candidateCount", json!(10001)),
        ("acceptedLimit", json!(0)),
        ("acceptedLimit", json!(10001)),
        ("blockId", json!("../escape")),
    ] {
        let mut v = fixture();
        v[field] = bad;
        assert_eq!(
            journal
                .execute(&serde_json::to_vec(&v).unwrap())
                .unwrap_err()
                .status,
            400
        );
    }
    let mut bad_hash = fixture();
    bad_hash["job"]["generation"]["templateHash"] = json!("0".repeat(64));
    assert_eq!(
        journal
            .execute(&serde_json::to_vec(&bad_hash).unwrap())
            .unwrap_err()
            .status,
        400
    );
    let mut many = fixture();
    many["job"]["generation"]["count"] = json!(2);
    assert_eq!(
        journal
            .execute(&serde_json::to_vec(&many).unwrap())
            .unwrap_err()
            .status,
        400
    );
    std::fs::remove_dir_all(dir).unwrap();
}
#[tokio::test]
async fn worker_is_opt_in_and_uses_existing_private_transport_guards() {
    use mosaicforge_preview::http::router;
    assert_eq!(
        call(
            router("test-token".into(), "127.0.0.1:4771".into()),
            &fixture()
        )
        .await
        .0,
        404
    );
    let dir = std::env::temp_dir().join(format!("mosaic-block-private-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let app = router_with_journal("test-token".into(), "127.0.0.1:4771".into(), &dir).unwrap();
    for (header, value, expected) in [
        ("authorization", "Bearer wrong", 401),
        ("host", "elsewhere", 403),
        ("origin", "https://example.com", 403),
        ("content-type", "text/plain", 415),
        ("content-length", "33554433", 413),
    ] {
        let mut request = Request::post("/internal/v1/blocks")
            .header("host", "127.0.0.1:4771")
            .header("authorization", "Bearer test-token")
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();
        request.headers_mut().insert(
            axum::http::HeaderName::from_bytes(header.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
        assert_eq!(
            app.clone()
                .oneshot(request)
                .await
                .unwrap()
                .status()
                .as_u16(),
            expected
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn checksums_are_over_exact_request_bytes_and_sorted_key_records() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    use sha2::{Digest, Sha256};
    let dir = std::env::temp_dir().join(format!("mosaic-block-hash-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let journal = Journal::open(&dir, JournalLimits::default()).unwrap();
    let bytes = serde_json::to_vec(&fixture()).unwrap();
    let response = journal.execute(&bytes).unwrap();
    let r: Value = serde_json::from_slice(&response).unwrap();
    assert_eq!(r["requestHash"], format!("{:x}", Sha256::digest(&bytes)));
    assert_eq!(
        r["recordsSha256"],
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&r["records"]).unwrap())
        )
    );
    let mut spaced = bytes.clone();
    spaced.push(b' ');
    assert_eq!(journal.execute(&spaced).unwrap_err().status, 409);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn journal_excludes_second_owner_and_refuses_symlink_roots() {
    use mosaicforge_preview::durable::{Journal, JournalLimits};
    let dir = std::env::temp_dir().join(format!("mosaic-block-owner-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let first = Journal::open(&dir, JournalLimits::default()).unwrap();
    assert!(
        Journal::open(&dir, JournalLimits::default()).is_err(),
        "two executors could exceed the same disk quota"
    );
    drop(first);
    assert!(Journal::open(&dir, JournalLimits::default()).is_ok());
    #[cfg(unix)]
    {
        let link = dir.with_extension("symlink");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&dir, &link).unwrap();
        assert!(Journal::open(&link, JournalLimits::default()).is_err());
        std::fs::remove_file(link).unwrap();
    }
    std::fs::remove_dir_all(dir).unwrap();
}
